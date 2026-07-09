use crate::color::Color;
use crate::ship::module::ShipModuleKind;
use glam::{IVec2, UVec2};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub modules: Modules,
    pub camera: Camera,
    pub starfield: Starfield,
    pub splash: Splash,
    pub editor: Editor,
    pub thrusters: Thrusters,
    pub flight: Flight,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ModuleDef {
    pub mass: f32,
    pub power: f32,
    pub thrust: f32,
    pub color: Color,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Modules {
    pub hull: ModuleDef,
    pub thruster: ModuleDef,
    pub reactor: ModuleDef,
}

impl Modules {
    pub fn def(&self, kind: ShipModuleKind) -> &ModuleDef {
        match kind {
            ShipModuleKind::Hull => &self.hull,
            ShipModuleKind::Thruster => &self.thruster,
            ShipModuleKind::Reactor => &self.reactor,
        }
    }
}

// Module colours loosely follow https://lospec.com/palette-list/endesga-32
impl Default for Modules {
    fn default() -> Self {
        Self {
            hull: ModuleDef {
                mass: 1.0,
                power: 0.0,
                thrust: 0.0,
                color: Color::rgb_hex(0x8B9BB4),
            },
            thruster: ModuleDef {
                mass: 2.0,
                power: -5.0,
                thrust: 100.0,
                color: Color::rgb_hex(0xF77622),
            },
            reactor: ModuleDef {
                mass: 4.0,
                power: 20.0,
                thrust: 0.0,
                color: Color::rgb_hex(0x3E8948),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Camera {
    /// Orthographic scale on entering the world; smaller = more zoomed in.
    pub default_scale: f32,
    pub min_scale: f32,
    pub max_scale: f32,
    /// Fraction of scale changed per unit of scroll.
    pub zoom_sensitivity: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            default_scale: 0.05,
            min_scale: 0.02,
            max_scale: 0.12,
            zoom_sensitivity: 0.1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct StarLayer {
    /// Higher = further away = drifts less across the screen as the camera moves.
    pub parallax: f32,
    pub z: f32,
    pub count: u32,
    pub min_size: f32,
    pub max_size: f32,
    pub color: Color,
    pub min_alpha: f32,
    pub max_alpha: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Starfield {
    /// World-space size of the wrapping tile each particle lives in. Must exceed
    /// the viewport at max zoom-out so the camera-centred window is always full.
    pub tile: f32,
    /// Depth layers, near to far.
    pub layers: Vec<StarLayer>,
}

impl Default for Starfield {
    fn default() -> Self {
        Self {
            tile: 512.0,
            layers: vec![
                // Dust: closest, drifts the most, small and faint grey.
                StarLayer {
                    parallax: 0.32,
                    z: -5.0,
                    count: 650,
                    min_size: 0.08,
                    max_size: 0.22,
                    color: Color::rgb(166, 166, 179),
                    min_alpha: 0.12,
                    max_alpha: 0.4,
                },
                // Debris / asteroids: mid depth, bigger, solid grey.
                StarLayer {
                    parallax: 0.6,
                    z: -10.0,
                    count: 220,
                    min_size: 0.5,
                    max_size: 1.2,
                    color: Color::rgb(128, 128, 140),
                    min_alpha: 0.7,
                    max_alpha: 1.0,
                },
                // Stars: far, barely moves, small and white.
                StarLayer {
                    parallax: 0.93,
                    z: -20.0,
                    count: 800,
                    min_size: 0.1,
                    max_size: 0.28,
                    color: Color::rgb(255, 255, 255),
                    min_alpha: 0.75,
                    max_alpha: 1.0,
                },
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Splash {
    pub secs: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Editor {
    pub canvas_origin: IVec2,
    pub canvas_size: UVec2,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            canvas_origin: IVec2::new(-32, -32),
            canvas_size: UVec2::new(64, 64),
        }
    }
}

/// Flight-model tuning that turns the raw Newtonian dynamics into something that
/// feels good to fly: caps top speed, bleeds off drift so ships don't coast
/// forever, and softens how hard mass drags on acceleration. The speed/damping
/// assists switch off at `0`; `mass_response` is neutral at `1.0`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Flight {
    /// Hard cap on linear speed, in cells/sec.
    pub max_speed: f32,
    /// Hard cap on angular speed, in rad/sec.
    pub max_angular_speed: f32,
    /// Fraction of linear velocity bled off per second. Also sets a natural top
    /// speed of `thrust_accel / linear_damping`, reached below `max_speed`.
    pub linear_damping: f32,
    /// Fraction of angular velocity bled off per second.
    pub angular_damping: f32,
    /// Exponent on mass/inertia in the acceleration response: `accel = force /
    /// mass^mass_response`. `1.0` is honest Newtonian; below `1.0`, dead weight
    /// (hull, reactors) costs less acceleration, so structured ships stay peppy
    /// while heavier still means slower. Applies to turning too.
    pub mass_response: f32,
}

impl Default for Flight {
    fn default() -> Self {
        Self {
            max_speed: 100.0,
            max_angular_speed: 10.0,
            linear_damping: 0.25,
            angular_damping: 0.8,
            mass_response: 0.8,
        }
    }
}

/// Thruster exhaust flame visuals. A flame renders on each firing port, ramping
/// its length with the port's live throttle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Thrusters {
    pub flame_color: Color,
    /// Flame length at full throttle, in cell units (scales with throttle).
    pub flame_length: f32,
    pub flame_width: f32,
    /// Throttle below which no flame is drawn (avoids flicker near zero).
    pub min_throttle: f32,
    /// Turbulent flicker amplitude as a fraction of length (0 = steady flame).
    pub flicker: f32,
    /// Spool-down time constant in seconds: how long a flame takes to trail off
    /// after its thruster stops firing. Spool-up is instant.
    pub spool_secs: f32,
}

impl Default for Thrusters {
    fn default() -> Self {
        Self {
            flame_color: Color::rgb(255, 190, 60),
            flame_length: 2.2,
            flame_width: 0.8,
            min_throttle: 0.03,
            flicker: 0.15,
            spool_secs: 0.1,
        }
    }
}
