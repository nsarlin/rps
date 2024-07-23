use crate::actions::Action;
use crate::loading::TextureAssets;
use crate::GameState;
use bevy::prelude::*;
use rand::Rng;

pub struct PlayerPlugin;

const PLAYER_COUNT: usize = 10;

#[derive(Component)]
pub struct Player;

trait Rps {
    fn texture(textures: &TextureAssets) -> Handle<Image>;
}

#[derive(Component, Default)]
pub struct Rock;

impl Rps for Rock {
    fn texture(textures: &TextureAssets) -> Handle<Image> {
        textures.rock.clone()
    }
}

#[derive(Component, Default)]
pub struct Paper;

impl Rps for Paper {
    fn texture(textures: &TextureAssets) -> Handle<Image> {
        textures.paper.clone()
    }
}

#[derive(Component, Default)]
pub struct Scissors;

impl Rps for Scissors {
    fn texture(textures: &TextureAssets) -> Handle<Image> {
        textures.scissors.clone()
    }
}

const PLAYER_SIZE: f32 = 52.;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_rps)
            .add_systems(
                Update,
                (
                    move_player,
                    handle_collides::<Rock, Scissors>,
                    handle_collides::<Scissors, Paper>,
                    handle_collides::<Paper, Rock>,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn spawn_single_rps<T: Component + Rps + Default>(
    commands: &mut Commands,
    window: &Window,
    textures: &TextureAssets,
    spawned: &mut Vec<Vec3>,
) {
    let screen_size_x = window.resolution.width();
    let screen_size_y = window.resolution.width();

    let mut rng = rand::thread_rng();
    let radius = PLAYER_SIZE / 2.;

    let pos = loop {
        let pos_x = rng.gen_range(radius..(screen_size_x - radius));
        let pos_y = rng.gen_range(radius..(screen_size_y - radius));

        let pos = Vec3::new(
            pos_x - (screen_size_x / 2.),
            pos_y - (screen_size_y / 2.),
            1.,
        );

        if is_inside(pos, window)
            && !spawned
                .iter()
                .any(|spawned_pos| is_colliding(*spawned_pos, pos))
        {
            break pos;
        }
    };

    spawned.push(pos);

    commands
        .spawn(SpriteBundle {
            texture: T::texture(textures),
            transform: Transform::from_translation(pos).with_scale(Vec3::new(0.1, 0.1, 0.)),
            ..Default::default()
        })
        .insert(T::default())
        .insert(Action::default())
        .insert(Player);
}

fn spawn_rps(mut commands: Commands, windows: Query<&Window>, textures: Res<TextureAssets>) {
    let window = windows.single();
    let mut spawned = Vec::new();

    for _ in 0..PLAYER_COUNT {
        spawn_single_rps::<Rock>(&mut commands, window, &textures, &mut spawned);
        spawn_single_rps::<Paper>(&mut commands, window, &textures, &mut spawned);
        spawn_single_rps::<Scissors>(&mut commands, window, &textures, &mut spawned);
    }
}

fn is_inside(pos: Vec3, window: &Window) -> bool {
    let screen_size_x = window.resolution.width();
    let screen_size_y = window.resolution.height();
    let radius = PLAYER_SIZE / 2.;

    let max_x = screen_size_x / 2. - radius;
    let max_y = screen_size_y / 2. - radius;

    let min_x = -screen_size_x / 2. + radius;
    let min_y = -screen_size_y / 2. + radius;

    pos.x > min_x && pos.x < max_x && pos.y > min_y && pos.y < max_y
}

fn move_player(
    time: Res<Time>,
    mut player_query: Query<(&mut Transform, &Action), With<Player>>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    let speed = 150.;

    for (mut player_transform, action) in player_query.iter_mut() {
        let movement = if let Some(movement) = action.movement {
            let wanted = Vec3::new(
                movement.x * speed * time.delta_seconds(),
                movement.y * speed * time.delta_seconds(),
                0.,
            );

            if is_inside(player_transform.translation + wanted, window) {
                wanted
            } else {
                Vec3::ZERO
            }
        } else {
            Vec3::ZERO
        };

        player_transform.translation += movement;
    }
}

fn is_colliding(src: Vec3, target: Vec3) -> bool {
    src.distance(target) < PLAYER_SIZE
}

fn handle_collides<S: Component + Rps + Default, T: Component>(
    mut commands: Commands,
    src_query: Query<&Transform, With<S>>,
    target_query: Query<(Entity, &Transform), With<T>>,
    textures: Res<TextureAssets>,
) {
    let mut deads = Vec::new();
    for src_pos in src_query.iter() {
        for (tgt, tgt_pos) in target_query.iter() {
            if !deads.contains(&tgt) {
                if is_colliding(src_pos.translation, tgt_pos.translation) {
                    commands.entity(tgt).despawn();

                    commands
                        .spawn(SpriteBundle {
                            texture: S::texture(&textures),
                            transform: *tgt_pos,
                            ..Default::default()
                        })
                        .insert(S::default())
                        .insert(Action::default())
                        .insert(Player);

                    deads.push(tgt);
                }
            }
        }
    }
}
