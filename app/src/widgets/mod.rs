use bevy_egui::egui;
pub use module_palette::ModulePalette;
pub use ship_stats::ShipStats;
pub use title_tag::TitleTag;

mod module_palette;
mod ship_stats;
mod title_tag;

pub fn viewport_root(ctx: &egui::Context, id: impl std::hash::Hash + std::fmt::Debug) -> egui::Ui {
    egui::Ui::new(
        ctx.clone(),
        egui::Id::new(id),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    )
}
