use crate::actions::Action;
use crate::loading::TextureAssets;
use crate::GameState;
use bevy::prelude::*;
use rand::Rng;

pub struct PlayerPlugin;

const PLAYER_COUNT: usize = 10;
const PLAYER_SIZE: f32 = 52.;

#[derive(Component)]
pub struct Player;

pub trait Rps {
    type Target: Rps + Component;
    type Predator: Rps + Component;
    fn texture(textures: &TextureAssets) -> Handle<Image>;
    fn flee_radius() -> f32;
    fn attack_radius() -> f32;
}

#[derive(Component, Default)]
pub struct Rock;

impl Rps for Rock {
    fn texture(textures: &TextureAssets) -> Handle<Image> {
        textures.rock.clone()
    }

    fn flee_radius() -> f32 {
        500.
    }

    fn attack_radius() -> f32 {
        300.
    }

    type Target = Scissors;

    type Predator = Paper;
}

#[derive(Component, Default)]
pub struct Paper;

impl Rps for Paper {
    fn texture(textures: &TextureAssets) -> Handle<Image> {
        textures.paper.clone()
    }

    fn flee_radius() -> f32 {
        400.
    }

    fn attack_radius() -> f32 {
        400.
    }

    type Target = Rock;

    type Predator = Scissors;
}

#[derive(Component, Default)]
pub struct Scissors;

impl Rps for Scissors {
    fn texture(textures: &TextureAssets) -> Handle<Image> {
        textures.scissors.clone()
    }

    fn flee_radius() -> f32 {
        300.
    }

    fn attack_radius() -> f32 {
        500.
    }

    type Target = Paper;

    type Predator = Rock;
}

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_rps)
            .add_systems(
                Update,
                (
                    move_player,
                    handle_collides::<Rock>,
                    handle_collides::<Scissors>,
                    handle_collides::<Paper>,
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

fn clip_inside(pos: Vec3, window: &Window) -> Vec3 {
    let screen_size_x = window.resolution.width();
    let screen_size_y = window.resolution.height();
    let radius = PLAYER_SIZE / 2.;

    let max_x = screen_size_x / 2. - radius;
    let max_y = screen_size_y / 2. - radius;

    let min_x = -screen_size_x / 2. + radius;
    let min_y = -screen_size_y / 2. + radius;

    let pos_x = if pos.x < min_x {
        min_x
    } else if pos.x > max_x {
        max_x
    } else {
        pos.x
    };

    let pos_y = if pos.y < min_y {
        min_y
    } else if pos.y > max_y {
        max_y
    } else {
        pos.y
    };

    Vec3::new(pos_x, pos_y, 0.)
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
            Vec3::new(
                movement.x * speed * time.delta_seconds(),
                movement.y * speed * time.delta_seconds(),
                0.,
            )
        } else {
            Vec3::ZERO
        };

        player_transform.translation = clip_inside(player_transform.translation + movement, window);
    }
}

fn is_colliding(src: Vec3, target: Vec3) -> bool {
    src.distance(target) < PLAYER_SIZE
}

fn handle_collides<S: Component + Rps + Default>(
    mut commands: Commands,
    src_query: Query<&Transform, With<S>>,
    target_query: Query<(Entity, &Transform), With<S::Target>>,
    textures: Res<TextureAssets>,
) {
    let mut deads = Vec::new();
    for src_pos in src_query.iter() {
        for (tgt, tgt_pos) in target_query.iter() {
            if !deads.contains(&tgt) && is_colliding(src_pos.translation, tgt_pos.translation) {
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
