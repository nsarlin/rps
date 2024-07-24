use bevy::prelude::*;
use rand::distributions::{Distribution, Standard};
use rand::Rng;

use crate::player::{Paper, Rock, Rps, Scissors};
use crate::GameState;

pub struct ActionsPlugin;

// This plugin listens for keyboard input and converts the input into Actions
// Actions can then be used as a resource in other systems to act on the player input.
impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                set_movement_actions::<Rock>,
                set_movement_actions::<Paper>,
                set_movement_actions::<Scissors>,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .insert_resource(Time::<Fixed>::from_hz(5.0));
    }
}

#[derive(Default, Component)]
pub struct Action {
    pub movement: Option<Vec2>,
}

#[derive(Debug)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Distribution<Direction> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        match rng.gen_range(0..=3) {
            // rand 0.8
            0 => Direction::Up,
            1 => Direction::Right,
            2 => Direction::Down,
            _ => Direction::Left,
        }
    }
}

impl From<Direction> for Vec2 {
    fn from(value: Direction) -> Self {
        match value {
            Direction::Up => Vec2::new(1., 0.),
            Direction::Right => Vec2::new(0., 1.),
            Direction::Down => Vec2::new(-1., 0.),
            Direction::Left => Vec2::new(0., -1.),
        }
    }
}

enum MoveAction {
    Random,
    Flee(Vec3),
    Attack(Vec3),
}

pub fn set_movement_actions<S: Component + Rps>(
    mut actions: Query<(&Transform, &mut Action), With<S>>,
    target_query: Query<&Transform, With<S::Target>>,
    predator_query: Query<&Transform, With<S::Predator>>,
) {
    for (pos, mut action) in actions.iter_mut() {
        let nearest_tgt = target_query
            .iter()
            .map(|tgt_pos| {
                (
                    tgt_pos,
                    pos.translation.distance_squared(tgt_pos.translation),
                )
            })
            .reduce(|(min_pos, min_dist), (pos, dist)| {
                if dist < min_dist {
                    (pos, dist)
                } else {
                    (min_pos, min_dist)
                }
            })
            .and_then(|(pos, dist)| {
                if dist < S::attack_radius().powi(2) {
                    Some((pos, dist))
                } else {
                    None
                }
            });

        let nearest_pred = predator_query
            .iter()
            .map(|tgt_pos| {
                (
                    tgt_pos,
                    pos.translation.distance_squared(tgt_pos.translation),
                )
            })
            .reduce(|(min_pos, min_dist), (pos, dist)| {
                if dist < min_dist {
                    (pos, dist)
                } else {
                    (min_pos, min_dist)
                }
            })
            .and_then(|(pos, dist)| {
                if dist < S::flee_radius().powi(2) {
                    Some((pos, dist))
                } else {
                    None
                }
            });

        let move_action = match (nearest_tgt, nearest_pred) {
            (None, None) => MoveAction::Random,
            (Some((tgt_pos, _)), None) => MoveAction::Attack(tgt_pos.translation),
            (None, Some((pred_pos, _))) => MoveAction::Flee(pred_pos.translation),
            (Some((tgt_pos, tgt_dist)), Some((pred_pos, pred_dist))) => {
                if tgt_dist < pred_dist {
                    MoveAction::Attack(tgt_pos.translation)
                } else {
                    MoveAction::Flee(pred_pos.translation)
                }
            }
        };

        match move_action {
            MoveAction::Random => {
                let movement = Vec2::new(rand::random(), rand::random()) - Vec2::new(0.5, 0.5);
                action.movement = Some(movement.normalize_or_zero());
            }
            MoveAction::Flee(pred_pos) => {
                let movement = pos.translation - pred_pos;
                action.movement = Some(movement.truncate().normalize());
            }
            MoveAction::Attack(tgt_pos) => {
                let movement = tgt_pos - pos.translation;
                action.movement = Some(movement.truncate().normalize());
            }
        };
    }
}
