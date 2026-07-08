mod plugins;
mod scenes;
mod state;
mod widgets;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

use plugins::{CameraPlugin, GamePlugin, StarfieldPlugin, ThemePlugin};
use scenes::{MainMenuPlugin, ShipEditorPlugin, SplashPlugin, WorldPlugin};
use state::AppState;

fn main() {
    let mut app = App::new();

    #[cfg(not(debug_assertions))]
    app.add_plugins(bevy_embedded_assets::EmbeddedAssetPlugin {
        mode: bevy_embedded_assets::PluginMode::ReplaceDefault,
    });

    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Pixel Spaceships".into(),
                    ..default()
                }),
                ..default()
            }),
    )
    .add_plugins(EguiPlugin::default())
    .init_state::<AppState>()
    .add_plugins((CameraPlugin, GamePlugin, StarfieldPlugin, ThemePlugin))
    .add_plugins((SplashPlugin, MainMenuPlugin, ShipEditorPlugin, WorldPlugin))
    .run();
}
