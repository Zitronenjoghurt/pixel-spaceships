use bevy_egui::egui::{Response, Ui, Widget};
use egui_phosphor::regular;

pub struct TitleTag;

impl Widget for TitleTag {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            ui.strong(format!("{} Pixel Spaceships", regular::ROCKET_LAUNCH));
            ui.weak(concat!("v", env!("CARGO_PKG_VERSION")));
        })
        .response
    }
}
