use bevy_egui::egui::{self, Color32, Response, RichText, Ui, Widget};
use egui_phosphor::regular;
use pixel_spaceships_core::ship::Ship;
use pixel_spaceships_core::ship::thrust::ShipThrust;

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

            ui.separator();
            thrust(ui, &stats.thrust);
        })
        .response
    }
}

fn thrust(ui: &mut Ui, thrust: &ShipThrust) {
    ui.heading("Thrust");

    ui.weak("Directional");
    egui::Grid::new("thrust_compass")
        .num_columns(3)
        .spacing([16.0, 6.0])
        .show(ui, |ui| {
            ui.label("");
            arrow(ui, regular::ARROW_FAT_UP, thrust.north);
            ui.label("");
            ui.end_row();

            arrow(ui, regular::ARROW_FAT_LEFT, thrust.west);
            ui.label("");
            arrow(ui, regular::ARROW_FAT_RIGHT, thrust.east);
            ui.end_row();

            ui.label("");
            arrow(ui, regular::ARROW_FAT_DOWN, thrust.south);
            ui.label("");
            ui.end_row();
        });

    ui.add_space(4.0);
    ui.weak("Angular");
    ui.horizontal(|ui| {
        arrow(ui, regular::ARROW_COUNTER_CLOCKWISE, thrust.ccw);
        ui.add_space(12.0);
        arrow(ui, regular::ARROW_CLOCKWISE, thrust.cw);
    });
}

fn arrow(ui: &mut Ui, icon: &str, value: f32) {
    let color = if value > 0.0 {
        ui.visuals().strong_text_color()
    } else {
        ui.visuals().weak_text_color()
    };
    ui.allocate_ui(egui::vec2(44.0, 42.0), |ui| {
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(icon).size(20.0).color(color));
            ui.monospace(RichText::new(format!("{value:.1}")).color(color));
        });
    });
}
