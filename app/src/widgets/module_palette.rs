use bevy_egui::egui::{Response, Ui, Widget};
use pixel_spaceships_core::ship::module::ShipModuleKind;
use strum::IntoEnumIterator;

pub struct ModulePalette<'a> {
    selected: &'a mut ShipModuleKind,
}

impl<'a> ModulePalette<'a> {
    pub fn new(selected: &'a mut ShipModuleKind) -> Self {
        Self { selected }
    }
}

impl Widget for ModulePalette<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.heading("Modules");
            for kind in ShipModuleKind::iter() {
                let selected = *self.selected == kind;
                if ui.selectable_label(selected, kind.def().name).clicked() {
                    *self.selected = kind;
                }
            }
        })
        .response
    }
}
