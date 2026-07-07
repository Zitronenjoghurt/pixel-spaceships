use crate::plugins::ActiveShip;
use crate::state::{AppState, EditorTool};
use crate::widgets::{ModulePalette, ShipCanvas, ShipStats, TitleTag, viewport_root};
use bevy::asset::RenderAssetUsages;
use bevy::image::{Image, ImageSampler};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, EguiTextureHandle, egui};
use egui_phosphor::regular;
use pixel_spaceships_core::ship::module::ShipModuleKind;

/// Fixed editing canvas: a stable region so click -> cell is a constant transform.
const CANVAS_ORIGIN: IVec2 = IVec2::new(-32, -32);
const CANVAS_SIZE: UVec2 = UVec2::new(64, 64);

pub struct ShipEditorPlugin;

impl Plugin for ShipEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<EditorTool>()
            .init_resource::<SelectedModule>()
            .add_systems(OnEnter(AppState::ShipEditor), setup_editor)
            .add_systems(OnExit(AppState::ShipEditor), teardown_editor)
            .add_systems(
                Update,
                sync_canvas
                    .run_if(in_state(AppState::ShipEditor))
                    .run_if(resource_exists::<EditorCanvas>),
            )
            .add_systems(
                EguiPrimaryContextPass,
                editor_ui.run_if(in_state(AppState::ShipEditor)),
            );
    }
}

#[derive(Resource, Default)]
struct SelectedModule(ShipModuleKind);

#[derive(Resource)]
struct EditorCanvas {
    image: Handle<Image>,
    texture: egui::TextureId,
}

fn setup_editor(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut contexts: EguiContexts,
    ship: Res<ActiveShip>,
) {
    let mut image = Image::new_fill(
        Extent3d {
            width: CANVAS_SIZE.x,
            height: CANVAS_SIZE.y,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    image.data = Some(ship.0.rasterize(CANVAS_ORIGIN, CANVAS_SIZE).pixels);

    let image = images.add(image);
    let texture = contexts.add_image(EguiTextureHandle::Strong(image.clone()));

    commands.insert_resource(EditorCanvas { image, texture });
    commands.spawn((Name::new("EditorRoot"), DespawnOnExit(AppState::ShipEditor)));
}

fn teardown_editor(
    mut commands: Commands,
    canvas: Res<EditorCanvas>,
    mut contexts: EguiContexts,
    mut images: ResMut<Assets<Image>>,
) {
    contexts.remove_image(&canvas.image);
    images.remove(&canvas.image);
    commands.remove_resource::<EditorCanvas>();
}

fn sync_canvas(
    ship: Res<ActiveShip>,
    canvas: Res<EditorCanvas>,
    mut images: ResMut<Assets<Image>>,
) {
    if !ship.is_changed() {
        return;
    }
    let raster = ship.0.rasterize(CANVAS_ORIGIN, CANVAS_SIZE);
    if let Some(mut image) = images.get_mut(&canvas.image) {
        image.data = Some(raster.pixels);
    }
}

fn editor_ui(
    mut contexts: EguiContexts,
    mut selected: ResMut<SelectedModule>,
    mut ship: ResMut<ActiveShip>,
    canvas: Res<EditorCanvas>,
    tool: Res<State<EditorTool>>,
    mut next_tool: ResMut<NextState<EditorTool>>,
    mut next_app: ResMut<NextState<AppState>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
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
        .default_size(190.0)
        .show(&mut root, |ui| {
            ui.add(ShipStats::new(&ship.0));
        });

    egui::CentralPanel::default().show(&mut root, |ui| {
        let mut target = None;
        ui.add(ShipCanvas::new(
            canvas.texture,
            CANVAS_ORIGIN,
            CANVAS_SIZE,
            &mut target,
        ));

        if let Some(cell) = target {
            match tool.get() {
                EditorTool::Place => ship.0.place(cell, selected.0),
                EditorTool::Erase => {
                    ship.0.remove(cell);
                }
                EditorTool::Inspect => {}
            }
        }
    });

    Ok(())
}
