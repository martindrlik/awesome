use bevy::prelude::*;

pub const BACKGROUND_COLOR: Color = Color::srgb(0., 0., 0.);
pub const FOREGROUND_COLOR: Color = Color::srgb(0.3, 0.3, 0.3);

pub const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
pub const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.1, 0.1, 0.1);
pub const PRESSED_BUTTON: Color = Color::srgb(0.2, 0.2, 0.2);

pub const PADDLE_COLOR: Color = FOREGROUND_COLOR;
pub const BALL_COLOR: Color = FOREGROUND_COLOR;
pub const BRICK_COLOR: Color = FOREGROUND_COLOR;
pub const WALL_COLOR: Color = FOREGROUND_COLOR;
pub const TEXT_COLOR: Color = FOREGROUND_COLOR;
pub const SCORE_COLOR: Color = FOREGROUND_COLOR;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GameState {
    #[default]
    Splash,
    Menu,
    Game,
    GameOver,
}

#[derive(Resource, Debug, Component, PartialEq, Eq, Clone, Copy)]
pub enum DisplayQuality {
    Low,
    Medium,
    High,
}

#[derive(Resource, Debug, Component, PartialEq, Eq, Clone, Copy)]
pub struct Volume(u32);

pub mod game;
pub mod game_over;
pub mod menu;
pub mod splash;
