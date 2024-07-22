use bevy::prelude::*;
use rand::distributions::{Distribution, Standard};
use rand::Rng;

use crate::GameState;

pub struct ActionsPlugin;

// This plugin listens for keyboard input and converts the input into Actions
// Actions can then be used as a resource in other systems to act on the player input.
impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            set_movement_actions.run_if(in_state(GameState::Playing)),
        )
        .insert_resource(Time::<Fixed>::from_hz(2.0));
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

pub fn set_movement_actions(mut actions: Query<&mut Action>) {
    for mut action in actions.iter_mut() {
        let direction: Direction = rand::random();
        let movement: Vec2 = direction.into();

        if movement != Vec2::ZERO {
            action.movement = Some(movement.normalize());
        } else {
            action.movement = None;
        }
    }
}
