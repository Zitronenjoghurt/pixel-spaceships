use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    Splash,
    MainMenu,
    ShipEditor,
    World,
}

#[derive(SubStates, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[source(AppState = AppState::ShipEditor)]
pub enum EditorTool {
    #[default]
    Place,
    Erase,
    Inspect,
}
