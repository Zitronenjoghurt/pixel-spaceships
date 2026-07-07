use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use egui_phosphor::regular;

use crate::state::AppState;
use crate::widgets::{TitleTag, viewport_root};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            menu_ui.run_if(in_state(AppState::MainMenu)),
        );
    }
}

fn menu_ui(mut contexts: EguiContexts, mut next: ResMut<NextState<AppState>>) -> Result {
    let ctx = contexts.ctx_mut()?;
    let mut root = viewport_root(ctx, "main_menu");

    egui::Panel::top("menu_top").show(&mut root, |ui| {
        ui.horizontal(|ui| ui.add(TitleTag));
    });

    egui::CentralPanel::default().show(&mut root, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(80.0);
            if ui
                .button(format!("{} Ship Editor", regular::WRENCH))
                .clicked()
            {
                next.set(AppState::ShipEditor);
            }
            if ui
                .button(format!("{} Launch", regular::ROCKET_LAUNCH))
                .clicked()
            {
                next.set(AppState::World);
            }
        });
    });
    Ok(())
}
