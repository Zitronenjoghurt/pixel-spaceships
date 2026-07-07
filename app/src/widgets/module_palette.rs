use bevy_egui::egui::{Response, Ui, Widget};
use pixel_spaceships_core::ModuleKind;

pub struct ModulePalette<'a> {
    selected: &'a mut ModuleKind,
}

impl<'a> ModulePalette<'a> {
    pub fn new(selected: &'a mut ModuleKind) -> Self {
        Self { selected }
    }
}

impl Widget for ModulePalette<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.heading("Modules");
            for kind in ModuleKind::ALL {
                let selected = *self.selected == kind;
                if ui.selectable_label(selected, kind.def().name).clicked() {
                    *self.selected = kind;
                }
            }
        })
        .response
    }
}
