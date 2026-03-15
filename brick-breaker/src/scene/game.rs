use bevy::{
    math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume},
    prelude::*,
};

use super::{
    BALL_COLOR, BRICK_COLOR, GameState, PADDLE_COLOR, SCORE_COLOR, TEXT_COLOR, WALL_COLOR,
};

const PADDLE_SIZE: Vec2 = Vec2::new(120.0, 20.0);
const GAP_BETWEEN_PADDLE_AND_FLOOR: f32 = 60.0;
const PADDLE_SPEED: f32 = 500.0;
const PADDLE_PADDING: f32 = 10.0;

const BALL_STARTING_POSITION: Vec3 = Vec3::new(0.0, -50.0, 1.0);
const BALL_DIAMETER: f32 = 30.;
const BALL_SPEED: f32 = 400.0;
const INITIAL_BALL_DIRECTION: Vec2 = Vec2::new(0.5, -0.5);

const WALL_THICKNESS: f32 = 10.0;
const LEFT_WALL: f32 = -450.;
const RIGHT_WALL: f32 = 450.;
const BOTTOM_WALL: f32 = -300.;
const TOP_WALL: f32 = 300.;

const BRICK_SIZE: Vec2 = Vec2::new(100., 30.);
const GAP_BETWEEN_PADDLE_AND_BRICKS: f32 = 270.0;
const GAP_BETWEEN_BRICKS: f32 = 5.0;
const GAP_BETWEEN_BRICKS_AND_CEILING: f32 = 20.0;
const GAP_BETWEEN_BRICKS_AND_SIDES: f32 = 20.0;

const SCOREBOARD_FONT_SIZE: f32 = 33.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);

pub fn game_plugin(app: &mut App) {
    app.insert_resource(Score(0))
        .add_systems(OnEnter(GameState::Game), game_setup)
        .add_systems(
            FixedUpdate,
            (
                (apply_velocity, move_bricks).chain(),
                move_paddle,
                check_for_collisions,
            )
                .chain()
                .run_if(in_state(GameState::Game)),
        )
        .add_systems(Update, update_scoreboard)
        .add_observer(play_collision_sound);
}

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Ball;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Event)]
struct BallCollided;

#[derive(Component)]
struct Brick;

#[derive(Resource, Deref)]
struct CollisionSound(Handle<AudioSource>);

#[derive(Component, Default)]
struct Collider;

#[derive(Component)]
#[require(Sprite, Transform, Collider)]
struct Wall;

enum WallLocation {
    Left,
    Right,
    Bottom,
    Top,
}

impl WallLocation {
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(LEFT_WALL, 0.),
            WallLocation::Right => Vec2::new(RIGHT_WALL, 0.),
            WallLocation::Bottom => Vec2::new(0., BOTTOM_WALL),
            WallLocation::Top => Vec2::new(0., TOP_WALL),
        }
    }

    fn size(&self) -> Vec2 {
        let arena_height = TOP_WALL - BOTTOM_WALL;
        let arena_width = RIGHT_WALL - LEFT_WALL;
        assert!(arena_height > 0.0);
        assert!(arena_width > 0.0);

        match self {
            WallLocation::Left | WallLocation::Right => {
                Vec2::new(WALL_THICKNESS, arena_height + WALL_THICKNESS)
            }
            WallLocation::Bottom | WallLocation::Top => {
                Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS)
            }
        }
    }
}

impl Wall {
    fn new(location: WallLocation) -> (Wall, Sprite, Transform) {
        (
            Wall,
            Sprite::from_color(WALL_COLOR, Vec2::ONE),
            Transform {
                translation: location.position().extend(0.0),
                scale: location.size().extend(1.0),
                ..default()
            },
        )
    }
}

#[derive(Resource, Deref, DerefMut)]
struct Score(usize);

#[derive(Component)]
struct ScoreboardUi;

#[derive(Component)]
struct OnGameScreen;

fn game_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let paddle_y = BOTTOM_WALL + GAP_BETWEEN_PADDLE_AND_FLOOR;
    commands.spawn((
        DespawnOnExit(GameState::Game),
        (
            Sprite::from_color(PADDLE_COLOR, Vec2::ONE),
            Transform {
                translation: Vec3::new(0.0, paddle_y, 0.0),
                scale: PADDLE_SIZE.extend(1.0),
                ..default()
            },
            Paddle,
            Collider,
        ),
    ));
    commands.spawn((
        DespawnOnExit(GameState::Game),
        (
            Mesh2d(meshes.add(Circle::default())),
            MeshMaterial2d(materials.add(BALL_COLOR)),
            Transform::from_translation(BALL_STARTING_POSITION)
                .with_scale(Vec2::splat(BALL_DIAMETER).extend(1.)),
            Ball,
            Velocity(INITIAL_BALL_DIRECTION.normalize() * BALL_SPEED),
        ),
    ));
    commands.spawn((
        DespawnOnExit(GameState::Game),
        Wall::new(WallLocation::Left),
    ));
    commands.spawn((
        DespawnOnExit(GameState::Game),
        Wall::new(WallLocation::Right),
    ));
    commands.spawn((
        DespawnOnExit(GameState::Game),
        Wall::new(WallLocation::Bottom),
    ));
    commands.spawn((DespawnOnExit(GameState::Game), Wall::new(WallLocation::Top)));
    commands.spawn((
        DespawnOnExit(GameState::Game),
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        OnGameScreen,
        children![(
            Text::new("Score: "),
            TextFont {
                font_size: SCOREBOARD_FONT_SIZE,
                ..default()
            },
            TextColor(TEXT_COLOR),
            ScoreboardUi,
            Node {
                position_type: PositionType::Absolute,
                top: SCOREBOARD_TEXT_PADDING,
                left: SCOREBOARD_TEXT_PADDING,
                ..default()
            },
            children![(
                TextSpan::default(),
                TextFont {
                    font_size: SCOREBOARD_FONT_SIZE,
                    ..default()
                },
                TextColor(SCORE_COLOR),
            )],
        ),],
    ));

    let ball_collision_sound = asset_server.load("sounds/breakout_collision.ogg");
    commands.insert_resource(CollisionSound(ball_collision_sound));

    let total_width_of_bricks = (RIGHT_WALL - LEFT_WALL) - 2. * GAP_BETWEEN_BRICKS_AND_SIDES;
    let bottom_edge_of_bricks = paddle_y + GAP_BETWEEN_PADDLE_AND_BRICKS;
    let total_height_of_bricks = TOP_WALL - bottom_edge_of_bricks - GAP_BETWEEN_BRICKS_AND_CEILING;

    assert!(total_width_of_bricks > 0.0);
    assert!(total_height_of_bricks > 0.0);

    let n_columns = (total_width_of_bricks / (BRICK_SIZE.x + GAP_BETWEEN_BRICKS)).floor() as usize;
    let n_rows = (total_height_of_bricks / (BRICK_SIZE.y + GAP_BETWEEN_BRICKS)).floor() as usize;
    let n_vertical_gaps = n_columns - 1;

    let center_of_bricks = (LEFT_WALL + RIGHT_WALL) / 2.0;
    let left_edge_of_bricks = center_of_bricks
        - (n_columns as f32 / 2.0 * BRICK_SIZE.x)
        - n_vertical_gaps as f32 / 2.0 * GAP_BETWEEN_BRICKS;

    let offset_x = left_edge_of_bricks + BRICK_SIZE.x / 2.;
    let offset_y = bottom_edge_of_bricks + BRICK_SIZE.y / 2.;

    for row in 0..n_rows {
        for column in 0..n_columns {
            let brick_position = Vec2::new(
                offset_x + column as f32 * (BRICK_SIZE.x + GAP_BETWEEN_BRICKS),
                offset_y + row as f32 * (BRICK_SIZE.y + GAP_BETWEEN_BRICKS),
            );

            commands.spawn((
                DespawnOnExit(GameState::Game),
                (
                    Sprite {
                        color: BRICK_COLOR,
                        ..default()
                    },
                    Transform {
                        translation: brick_position.extend(0.0),
                        scale: Vec3::new(BRICK_SIZE.x, BRICK_SIZE.y, 1.0),
                        ..default()
                    },
                    Brick,
                    Collider,
                ),
            ));
        }
    }
}

fn move_bricks(mut brick_query: Query<&mut Transform, With<Brick>>, time: Res<Time>) {
    for mut transform in &mut brick_query {
        transform.translation.y -= 10. * time.delta_secs();
    }
}

fn move_paddle(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut paddle_transform: Single<&mut Transform, With<Paddle>>,
    time: Res<Time>,
) {
    let mut direction = 0.0;

    if is_left_pressed(&keyboard_input) {
        direction -= 1.0;
    }

    if is_right_pressed(&keyboard_input) {
        direction += 1.0;
    }

    let new_paddle_position =
        paddle_transform.translation.x + direction * PADDLE_SPEED * time.delta_secs();

    let left_bound = LEFT_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE.x / 2.0 + PADDLE_PADDING;
    let right_bound = RIGHT_WALL - WALL_THICKNESS / 2.0 - PADDLE_SIZE.x / 2.0 - PADDLE_PADDING;

    paddle_transform.translation.x = new_paddle_position.clamp(left_bound, right_bound);
}

fn is_left_pressed(keyboard_input: &Res<ButtonInput<KeyCode>>) -> bool {
    return keyboard_input.pressed(KeyCode::ArrowLeft);
}

fn is_right_pressed(keyboard_input: &Res<ButtonInput<KeyCode>>) -> bool {
    return keyboard_input.pressed(KeyCode::ArrowRight);
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time.delta_secs();
        transform.translation.y += velocity.y * time.delta_secs();
    }
}

fn update_scoreboard(
    score: Res<Score>,
    score_root: Single<Entity, (With<ScoreboardUi>, With<Text>)>,
    mut writer: TextUiWriter,
) {
    *writer.text(*score_root, 1) = score.to_string();
}

fn check_for_collisions(
    mut commands: Commands,
    mut score: ResMut<Score>,
    ball_query: Single<(&mut Velocity, &Transform), With<Ball>>,
    paddle_query: Single<&Transform, With<Paddle>>,
    collider_query: Query<(Entity, &Transform, Option<&Brick>), With<Collider>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let (mut ball_velocity, ball_transform) = ball_query.into_inner();
    let paddle_transform = paddle_query.into_inner();

    for (collider_entity, collider_transform, maybe_brick) in &collider_query {
        let collider_bounding_box = Aabb2d::new(
            collider_transform.translation.truncate(),
            collider_transform.scale.truncate() / 2.,
        );
        let collision = ball_collision(
            BoundingCircle::new(ball_transform.translation.truncate(), BALL_DIAMETER / 2.),
            collider_bounding_box,
        );
        if maybe_brick.is_some() {
            let paddle_collision = ball_collision(
                BoundingCircle::new(paddle_transform.translation.truncate(), BALL_DIAMETER / 2.),
                collider_bounding_box,
            );
            if let Some(_paddle_collision) = paddle_collision {
                game_state.set(GameState::Menu);
            }
        }

        if let Some(collision) = collision {
            commands.trigger(BallCollided);
            **score += 1;

            if maybe_brick.is_some() {
                commands.entity(collider_entity).despawn();
            }

            let mut reflect_x = false;
            let mut reflect_y = false;

            match collision {
                Collision::Left => reflect_x = ball_velocity.x > 0.0,
                Collision::Right => reflect_x = ball_velocity.x < 0.0,
                Collision::Top => reflect_y = ball_velocity.y < 0.0,
                Collision::Bottom => reflect_y = ball_velocity.y > 0.0,
            }

            if reflect_x {
                ball_velocity.x = -ball_velocity.x;
            }

            if reflect_y {
                ball_velocity.y = -ball_velocity.y;
            }
        }
    }
}

fn play_collision_sound(
    _collided: On<BallCollided>,
    mut commands: Commands,
    sound: Res<CollisionSound>,
) {
    commands.spawn((AudioPlayer(sound.clone()), PlaybackSettings::DESPAWN));
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Collision {
    Left,
    Right,
    Top,
    Bottom,
}

fn ball_collision(ball: BoundingCircle, bounding_box: Aabb2d) -> Option<Collision> {
    if !ball.intersects(&bounding_box) {
        return None;
    }

    let closest = bounding_box.closest_point(ball.center());
    let offset = ball.center() - closest;
    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x < 0. {
            Collision::Left
        } else {
            Collision::Right
        }
    } else if offset.y > 0. {
        Collision::Top
    } else {
        Collision::Bottom
    };

    Some(side)
}
