use crate::plugins::GameConfig;
use crate::state::AppState;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use egui_phosphor::regular;

pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Splash), setup)
            .add_systems(
                EguiPrimaryContextPass,
                splash.run_if(in_state(AppState::Splash)),
            );
    }
}

#[derive(Resource)]
struct SplashTimer(Timer);

fn setup(mut commands: Commands, config: Res<GameConfig>, mut clear: ResMut<ClearColor>) {
    commands.insert_resource(SplashTimer(Timer::from_seconds(
        config.splash.secs,
        TimerMode::Once,
    )));
    clear.0 = Color::BLACK;
}

fn splash(
    mut contexts: EguiContexts,
    time: Res<Time>,
    mut timer: ResMut<SplashTimer>,
    mut clear: ResMut<ClearColor>,
    mut next: ResMut<NextState<AppState>>,
) -> Result {
    let t = timer.0.tick(time.delta()).fraction();
    let ctx = contexts.ctx_mut()?;
    let grey = ctx.style_of(ctx.theme()).visuals.panel_fill;
    let target = Color::srgba_u8(grey.r(), grey.g(), grey.b(), 255);
    clear.0 = LinearRgba::BLACK.mix(&target.into(), t).into();

    egui::Area::new("splash".into())
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new(format!("{} Pixel Spaceships", regular::ROCKET_LAUNCH))
                    .size(48.0)
                    .color(egui::Color32::from_white_alpha((t * 255.0) as u8)),
            );
        });

    if timer.0.is_finished() {
        next.set(AppState::MainMenu);
    }
    Ok(())
}
