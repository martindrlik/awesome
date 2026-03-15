use bevy::prelude::*;

mod scene;

use scene::{BACKGROUND_COLOR, GameState, game, menu, splash};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .add_systems(Startup, setup)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_plugins((splash::splash_plugin, menu::menu_plugin, game::game_plugin))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
