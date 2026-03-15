use bevy::prelude::*;

use crate::scene::GameState;

use super::TEXT_COLOR;

pub fn game_over_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::GameOver), game_over_setup);
    app.add_systems(Update, countdown.run_if(in_state(GameState::GameOver)));
}

#[derive(Component)]
struct OnGameOverScreen;

#[derive(Resource, Deref, DerefMut)]
struct GameOverTimer(Timer);

fn game_over_setup(mut commands: Commands, _asset_server: Res<AssetServer>) {
    commands.spawn((
        DespawnOnExit(GameState::GameOver),
        Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        OnGameOverScreen,
        children![(
            Text::new("GAME OVER!"),
            TextFont {
                font_size: 67.0,
                ..default()
            },
            TextColor(TEXT_COLOR),
            Node {
                margin: UiRect::all(px(50)),
                ..default()
            },
        ),],
    ));
    commands.insert_resource(GameOverTimer(Timer::from_seconds(2.0, TimerMode::Once)));
}

fn countdown(
    mut game_state: ResMut<NextState<GameState>>,
    time: Res<Time>,
    mut timer: ResMut<GameOverTimer>,
) {
    if timer.tick(time.delta()).is_finished() {
        game_state.set(GameState::Menu);
    }
}
