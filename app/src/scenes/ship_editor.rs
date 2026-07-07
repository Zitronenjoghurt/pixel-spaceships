use crate::plugins::ActiveShip;
use crate::state::{AppState, EditorTool};
use crate::widgets::{ModulePalette, ShipStats, TitleTag, viewport_root};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use egui_phosphor::regular;
use pixel_spaceships_core::ModuleKind;

pub struct ShipEditorPlugin;

impl Plugin for ShipEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<EditorTool>()
            .init_resource::<SelectedModule>()
            .add_systems(OnEnter(AppState::ShipEditor), setup_editor)
            .add_systems(
                EguiPrimaryContextPass,
                editor_ui.run_if(in_state(AppState::ShipEditor)),
            );
    }
}

#[derive(Resource, Default)]
struct SelectedModule(ModuleKind);

fn setup_editor(mut commands: Commands) {
    commands.spawn((Name::new("EditorRoot"), DespawnOnExit(AppState::ShipEditor)));
}

fn editor_ui(
    mut contexts: EguiContexts,
    mut selected: ResMut<SelectedModule>,
    ship: Res<ActiveShip>,
    tool: Res<State<EditorTool>>,
    mut next_tool: ResMut<NextState<EditorTool>>,
    mut next_app: ResMut<NextState<AppState>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    // One root Ui for the whole scene: the top bar and both side panels lay out
    // against each other, so nothing overlaps.
    let mut root = viewport_root(ctx, "ship_editor");

    egui::Panel::top("editor_top").show(&mut root, |ui| {
        ui.horizontal(|ui| {
            ui.add(TitleTag);
            ui.separator();
            if ui.button(format!("{} Menu", regular::HOUSE)).clicked() {
                next_app.set(AppState::MainMenu);
            }
            if ui
                .button(format!("{} Launch", regular::ROCKET_LAUNCH))
                .clicked()
            {
                next_app.set(AppState::World);
            }
        });
    });

    egui::Panel::left("editor_palette")
        .resizable(true)
        .show(&mut root, |ui| {
            ui.heading("Tool");
            for t in [EditorTool::Place, EditorTool::Erase, EditorTool::Inspect] {
                let icon = match t {
                    EditorTool::Place => regular::PENCIL_SIMPLE,
                    EditorTool::Erase => regular::ERASER,
                    EditorTool::Inspect => regular::MAGNIFYING_GLASS,
                };
                if ui
                    .selectable_label(*tool.get() == t, format!("{icon} {t:?}"))
                    .clicked()
                {
                    next_tool.set(t);
                }
            }
            ui.separator();

            ui.add(ModulePalette::new(&mut selected.0));
        });

    egui::Panel::right("editor_inspector")
        .resizable(true)
        .show(&mut root, |ui| {
            ui.add(ShipStats::new(&ship.0));
        });

    Ok(())
}
