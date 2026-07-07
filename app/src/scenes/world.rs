use crate::state::AppState;
use crate::widgets::{TitleTag, viewport_root};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use egui_phosphor::regular;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            world_hud.run_if(in_state(AppState::World)),
        );
    }
}

fn world_hud(mut contexts: EguiContexts, mut next: ResMut<NextState<AppState>>) -> Result {
    let ctx = contexts.ctx_mut()?;
    let mut root = viewport_root(ctx, "world");

    egui::Panel::top("world_top").show(&mut root, |ui| {
        ui.horizontal(|ui| {
            ui.add(TitleTag);
            ui.separator();
            if ui.button(format!("{} Editor", regular::WRENCH)).clicked() {
                next.set(AppState::ShipEditor);
            }
        });
    });
    Ok(())
}
