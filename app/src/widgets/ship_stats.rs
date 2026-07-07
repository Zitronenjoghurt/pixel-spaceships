use bevy_egui::egui::{Color32, Response, Ui, Widget};
use pixel_spaceships_core::Ship;

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
        ui.vertical(|ui| {
            ui.heading("Ship stats");
            ui.label(format!("Mass: {:.1}", self.ship.total_mass()));

            let power = self.ship.power_balance();
            let color = if power >= 0.0 {
                Color32::GREEN
            } else {
                Color32::RED
            };
            ui.colored_label(color, format!("Power: {power:+.1}"));

            ui.label(format!("Thrust: {:.1}", self.ship.total_thrust()));
        })
        .response
    }
}
