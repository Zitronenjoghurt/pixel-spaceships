use bevy_egui::egui::{Color32, Response, Ui, Widget};
use pixel_spaceships_core::ship::Ship;

pub struct ShipStats<'a> {
    ship: &'a Ship,
}

impl<'a> ShipStats<'a> {
    pub fn new(ship: &'a Ship) -> Self {
        Self { ship }
    }
}

impl Widget for ShipStats<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let stats = &self.ship.stats;
        ui.vertical(|ui| {
            ui.heading("Ship stats");
            ui.label(format!("Mass: {:.1}", stats.total_mass));

            let power = stats.power_balance;
            let color = if power >= 0.0 {
                Color32::GREEN
            } else {
                Color32::RED
            };
            ui.colored_label(color, format!("Power: {power:+.1}"));

            ui.label(format!("Inertia: {:.1}", stats.moment_of_inertia));
            ui.label(format!(
                "CoM: ({:.1}, {:.1})",
                stats.center_of_mass.x, stats.center_of_mass.y
            ));
        })
        .response
    }
}
