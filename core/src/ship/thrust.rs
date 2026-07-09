use crate::direction::Direction;
use glam::{IVec2, Vec2, Vec3};

/// How hard parasitic (off-axis) force/torque is penalized when allocating
/// thrust. Higher penalty = cleaner flight but demands better-balanced thruster
/// layouts.
const PARASITIC_PENALTY: f32 = 64.0;
/// Coordinate-descent sweeps for the rotation solve. This converges slowly under
/// the strong parasitic penalty: at a few hundred sweeps the allocation is still
/// ~15% short of optimal *and* order-sensitive (so a ship's stats would depend on
/// HashMap iteration order). It plateaus by ~2000; beyond that only f32 noise
/// moves. Runs only when a ship changes, so the cost is off the hot path.
const SWEEPS: usize = 2000;

/// A single thruster face and its full-throttle contribution, in ship-local
/// frame. Exhaust exits the side opposite `push`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThrustPort {
    /// Thruster cell this face belongs to.
    pub cell: IVec2,
    /// Cardinal direction the ship is pushed.
    pub push: Direction,
    /// Linear contribution at full throttle (`push * thrust`).
    pub force: Vec2,
    /// Angular contribution at full throttle; positive is counter-clockwise.
    pub torque: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Forward,
    Back,
    Right,
    Left,
    Ccw,
    Cw,
}

impl Command {
    pub const ALL: [Command; 6] = [
        Command::Forward,
        Command::Back,
        Command::Right,
        Command::Left,
        Command::Ccw,
        Command::Cw,
    ];

    pub const fn index(self) -> usize {
        match self {
            Command::Forward => 0,
            Command::Back => 1,
            Command::Right => 2,
            Command::Left => 3,
            Command::Ccw => 4,
            Command::Cw => 5,
        }
    }

    /// Desired wrench axis in normalized `(fx, fy, torque)` space (rotation only).
    fn axis(self) -> Vec3 {
        match self {
            Command::Ccw => Vec3::new(0.0, 0.0, 1.0),
            Command::Cw => Vec3::new(0.0, 0.0, -1.0),
            _ => Vec3::ZERO,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ShipThrust {
    pub east: f32,
    pub west: f32,
    pub north: f32,
    pub south: f32,
    pub cw: f32,
    pub ccw: f32,
}

#[derive(Debug, Clone)]
pub struct ThrustProfile {
    pub ports: Vec<ThrustPort>,
    /// Per-command, per-port throttle in `[0, 1]`.
    patterns: [Vec<f32>; 6],
    /// Achievable clean wrench magnitude per command, in real units.
    achievable: [f32; 6],
}

impl Default for ThrustProfile {
    fn default() -> Self {
        Self {
            ports: Vec::new(),
            patterns: std::array::from_fn(|_| Vec::new()),
            achievable: [0.0; 6],
        }
    }
}

impl ThrustProfile {
    pub fn solve(ports: Vec<ThrustPort>) -> Self {
        let mut patterns: [Vec<f32>; 6] = std::array::from_fn(|_| Vec::new());
        let mut achievable = [0.0f32; 6];

        for (cmd, push) in [
            (Command::Forward, Direction::North),
            (Command::Back, Direction::South),
            (Command::Right, Direction::East),
            (Command::Left, Direction::West),
        ] {
            let (throttles, on_axis) = allocate_translation(&ports, push);
            patterns[cmd.index()] = throttles;
            achievable[cmd.index()] = on_axis;
        }

        let force_scale = ports.iter().map(|p| p.force.length()).fold(1e-3, f32::max);
        let torque_scale = ports.iter().map(|p| p.torque.abs()).fold(1e-3, f32::max);
        let norm: Vec<Vec3> = ports
            .iter()
            .map(|p| {
                Vec3::new(
                    p.force.x / force_scale,
                    p.force.y / force_scale,
                    p.torque / torque_scale,
                )
            })
            .collect();
        for cmd in [Command::Ccw, Command::Cw] {
            let (throttles, on_axis) = allocate(&norm, cmd.axis());
            patterns[cmd.index()] = throttles;
            achievable[cmd.index()] = on_axis * torque_scale;
        }

        Self {
            ports,
            patterns,
            achievable,
        }
    }

    pub fn capability(&self) -> ShipThrust {
        ShipThrust {
            north: self.achievable[Command::Forward.index()],
            south: self.achievable[Command::Back.index()],
            east: self.achievable[Command::Right.index()],
            west: self.achievable[Command::Left.index()],
            ccw: self.achievable[Command::Ccw.index()],
            cw: self.achievable[Command::Cw.index()],
        }
    }

    pub fn resolve_into(&self, commands: &[f32; 6], throttles: &mut Vec<f32>) {
        throttles.clear();
        throttles.resize(self.ports.len(), 0.0);
        for (pattern, &intensity) in self.patterns.iter().zip(commands) {
            if intensity == 0.0 {
                continue;
            }
            for (throttle, &p) in throttles.iter_mut().zip(pattern) {
                *throttle += intensity * p;
            }
        }
        for throttle in throttles.iter_mut() {
            *throttle = throttle.clamp(0.0, 1.0);
        }
    }

    pub fn net(&self, throttles: &[f32]) -> (Vec2, f32) {
        let mut force = Vec2::ZERO;
        let mut torque = 0.0;
        for (port, &u) in self.ports.iter().zip(throttles) {
            force += port.force * u;
            torque += port.torque * u;
        }
        (force, torque)
    }
}

/// Exact thrust allocation for translation. Fires every port pushing `push`
/// (their force is purely along the axis) and cancels the net torque exactly by
/// throttling down the ports that shed the most torque per unit thrust first.
/// Returns the per-port throttle pattern and the achievable clean force.
fn allocate_translation(ports: &[ThrustPort], push: Direction) -> (Vec<f32>, f32) {
    let mut u = vec![0.0f32; ports.len()];
    let candidates: Vec<usize> = (0..ports.len())
        .filter(|&i| ports[i].push == push)
        .collect();
    if candidates.is_empty() {
        return (u, 0.0);
    }
    for &i in &candidates {
        u[i] = 1.0;
    }

    let net_torque: f32 = candidates.iter().map(|&i| ports[i].torque).sum();
    if net_torque.abs() > 1e-6 {
        let sign = net_torque.signum();
        let mut cut: Vec<usize> = candidates
            .iter()
            .copied()
            .filter(|&i| ports[i].torque.signum() == sign)
            .collect();
        cut.sort_by(|&a, &b| {
            let ra = ports[a].torque.abs() / ports[a].force.length();
            let rb = ports[b].torque.abs() / ports[b].force.length();
            rb.partial_cmp(&ra).unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut remaining = net_torque.abs();
        for &i in &cut {
            if remaining <= 0.0 {
                break;
            }
            let torque = ports[i].torque.abs();
            if torque <= remaining {
                u[i] = 0.0;
                remaining -= torque;
            } else {
                u[i] = 1.0 - remaining / torque;
                remaining = 0.0;
            }
        }
    }

    let achievable: f32 = candidates
        .iter()
        .map(|&i| u[i] * ports[i].force.length())
        .sum();
    (u, achievable)
}

/// Projected coordinate descent maximizing on-axis thrust while penalizing
/// off-axis (parasitic) wrench, over per-port throttles in `[0, 1]`. Returns the
/// throttle pattern and the achievable on-axis magnitude (normalized units).
fn allocate(norm: &[Vec3], axis: Vec3) -> (Vec<f32>, f32) {
    let n = norm.len();
    let mut u = vec![0.0f32; n];
    if n == 0 {
        return (u, 0.0);
    }

    let coef: Vec<f32> = norm.iter().map(|w| w.dot(axis)).collect();
    let perp: Vec<Vec3> = norm
        .iter()
        .zip(&coef)
        .map(|(w, c)| *w - axis * *c)
        .collect();

    let mut parasitic = Vec3::ZERO;
    for _ in 0..SWEEPS {
        for i in 0..n {
            let pi = perp[i];
            let len2 = pi.length_squared();
            let without = parasitic - pi * u[i];
            let next = if len2 < 1e-9 {
                // Pure on-axis port: fire fully iff it helps.
                if coef[i] > 0.0 { 1.0 } else { 0.0 }
            } else {
                ((coef[i] - 2.0 * PARASITIC_PENALTY * pi.dot(without))
                    / (2.0 * PARASITIC_PENALTY * len2))
                    .clamp(0.0, 1.0)
            };
            parasitic = without + pi * next;
            u[i] = next;
        }
    }

    let on_axis: f32 = u.iter().zip(&coef).map(|(ui, ci)| ui * ci).sum();
    (u, on_axis.max(0.0))
}
