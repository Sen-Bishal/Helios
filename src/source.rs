//! Source subsystem: Tin droplet generation and laser-plasma interaction
//! 
//! Simulates the generation of 13.5nm EUV light via laser-produced plasma

use bevy_ecs::prelude::*;
use glam::Vec3;
use rand::Rng;
use rand_distr::{Distribution, Normal};
use crate::units::{Position3D, Distance};
use crate::components::*;

/// State machine for tin droplet lifecycle
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropletState {
    /// Initial spherical droplet
    Spherical,
    /// Flattened by pre-pulse laser
    Pancaked,
    /// Ionized plasma state (emitting EUV)
    Plasma,
    /// Solid debris after plasma collapse
    Debris,
}

/// Configuration for the droplet generator
#[derive(Resource, Debug, Clone)]
pub struct DropletGeneratorConfig {
    /// Droplet generation frequency (Hz)
    pub frequency: f32,
    /// Time between droplets (seconds)
    pub period: f32,
    /// Initial droplet velocity (m/s)
    pub velocity: f32,
    /// Velocity jitter std deviation (m/s)
    pub velocity_jitter: f32,
    /// Droplet mass (kg)
    pub mass: f32,
    /// Initial droplet radius
    pub radius: Distance,
    /// Position where droplets are spawned
    pub spawn_position: Position3D,
    /// Direction vector (normalized)
    pub spawn_direction: Vec3,
}

impl Default for DropletGeneratorConfig {
    fn default() -> Self {
        Self {
            frequency: 50_000.0, // 50 kHz
            period: 1.0 / 50_000.0, // 20 microseconds
            velocity: 100.0, // ~100 m/s (hundreds of mph)
            velocity_jitter: 0.5, // 0.5 m/s std deviation
            mass: 5e-9, // ~5 nanograms of tin
            radius: Distance::from_micrometers(30), // 30 μm diameter droplet
            spawn_position: Position3D::new(
                Distance::from_millimeters(-50),
                Distance::ZERO,
                Distance::ZERO,
            ),
            spawn_direction: Vec3::X, // Travel along X-axis
        }
    }
}

/// Tracks timing for droplet generation
#[derive(Resource, Debug)]
pub struct DropletGeneratorState {
    /// Accumulated time since last droplet (seconds)
    pub time_accumulator: f32,
    /// Total droplets spawned
    pub droplet_count: u64,
}

impl Default for DropletGeneratorState {
    fn default() -> Self {
        Self {
            time_accumulator: 0.0,
            droplet_count: 0,
        }
    }
}

/// System that spawns tin droplets at regular intervals
pub fn droplet_generator_system(
    mut commands: Commands,
    time: Res<SimulationTime>,
    config: Res<DropletGeneratorConfig>,
    mut state: ResMut<DropletGeneratorState>,
) {
    state.time_accumulator += time.delta_seconds;

    // Spawn droplets for each period that has elapsed
    while state.time_accumulator >= config.period {
        state.time_accumulator -= config.period;
        state.droplet_count += 1;

        // Add Gaussian jitter to velocity for realism
        let mut rng = rand::thread_rng();
        let jitter_dist = Normal::new(0.0, config.velocity_jitter).unwrap();
        let velocity_with_jitter = config.velocity + jitter_dist.sample(&mut rng);

        // Spawn the droplet entity
        commands.spawn((
            Position(config.spawn_position),
            Velocity(config.spawn_direction * velocity_with_jitter),
            Mass(config.mass),
            DropletState::Spherical,
            CollisionShape::Sphere {
                radius: config.radius,
            },
            EntityType::TinDroplet,
            ThermalState::new(293.15, 0.001), // Tin at room temp, low heat capacity
        ));
    }
}

/// Laser beam component
#[derive(Component, Debug, Clone)]
pub struct LaserBeam {
    /// Laser power in Watts
    pub power: f32,
    /// Is this the pre-pulse (true) or main pulse (false)?
    pub is_prepulse: bool,
    /// Has this laser fired?
    pub has_fired: bool,
}

impl LaserBeam {
    pub const PRE_PULSE_POWER: f32 = 1_000.0; // 1 kW
    pub const MAIN_PULSE_POWER: f32 = 20_000.0; // 20 kW
}

/// Laser targeting system - tracks droplets and fires when aligned
#[derive(Resource, Debug)]
pub struct LaserTargetingSystem {
    /// Target position for laser focus
    pub focal_point: Position3D,
    /// Sensor delay (seconds) - time to detect and process droplet position
    pub sensor_delay: f32,
    /// Laser cooldown timer
    pub cooldown: f32,
}

impl Default for LaserTargetingSystem {
    fn default() -> Self {
        Self {
            focal_point: Position3D::zero(), // Center of vacuum chamber
            sensor_delay: 0.000_001, // 1 microsecond
            cooldown: 0.0,
        }
    }
}

/// System that fires lasers at droplets when they reach the focal point
pub fn laser_targeting_system(
    mut commands: Commands,
    time: Res<SimulationTime>,
    mut targeting: ResMut<LaserTargetingSystem>,
    droplets: Query<(Entity, &Position, &DropletState)>,
) {
    // Update cooldown
    if targeting.cooldown > 0.0 {
        targeting.cooldown -= time.delta_seconds;
        return;
    }

    // Find droplets near the focal point
    for (entity, pos, state) in droplets.iter() {
        let distance_to_focal = pos.0.distance_to(&targeting.focal_point);
        let threshold = Distance::from_millimeters(1); // 1mm targeting window

        if distance_to_focal < threshold {
            // Fire appropriate laser based on droplet state
            match state {
                DropletState::Spherical => {
                    // Fire pre-pulse
                    spawn_laser_pulse(&mut commands, pos.0, true);
                    targeting.cooldown = 0.000_005; // 5 microseconds between pulses
                }
                DropletState::Pancaked => {
                    // Fire main pulse
                    spawn_laser_pulse(&mut commands, pos.0, false);
                    targeting.cooldown = targeting.sensor_delay;
                }
                _ => {}
            }
        }
    }
}

/// Helper function to spawn a laser pulse entity
fn spawn_laser_pulse(commands: &mut Commands, target_pos: Position3D, is_prepulse: bool) {
    let power = if is_prepulse {
        LaserBeam::PRE_PULSE_POWER
    } else {
        LaserBeam::MAIN_PULSE_POWER
    };

    commands.spawn((
        Position(target_pos),
        LaserBeam {
            power,
            is_prepulse,
            has_fired: false,
        },
        EntityType::LaserBeam,
        Lifetime::new(0.000_01), // Laser pulse lasts 10 microseconds
    ));
}

/// Simulation time resource
#[derive(Resource, Debug)]
pub struct SimulationTime {
    /// Total elapsed time (seconds)
    pub total_seconds: f64,
    /// Delta time for this tick (seconds)
    pub delta_seconds: f32,
}

impl Default for SimulationTime {
    fn default() -> Self {
        Self {
            total_seconds: 0.0,
            delta_seconds: 1e-6, // 1 microsecond default tick
        }
    }
}

impl SimulationTime {
    pub fn tick(&mut self, delta: f32) {
        self.delta_seconds = delta;
        self.total_seconds += delta as f64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_droplet_period() {
        let config = DropletGeneratorConfig::default();
        assert_eq!(config.period, 0.00002); // 20 microseconds
        assert_eq!(config.frequency, 50_000.0);
    }

    #[test]
    fn test_laser_power_levels() {
        assert_eq!(LaserBeam::PRE_PULSE_POWER, 1_000.0);
        assert_eq!(LaserBeam::MAIN_PULSE_POWER, 20_000.0);
    }
}