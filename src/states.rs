use bevy::prelude::*;

#[allow(dead_code)]
#[derive(States, Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    MainMenu,
    #[default]
    InGame,
}

#[allow(dead_code)]
#[derive(SubStates, Debug, Default, Clone, PartialEq, Eq, Hash)]
#[source(AppState = AppState::InGame)]
pub enum GamePlayState {
    #[default]
    GeneratingMaze,
    Playing,
    LevelDone,
}

pub fn plugin(app: &mut App) {
    app.init_state::<AppState>()
        .add_sub_state::<GamePlayState>();
}
