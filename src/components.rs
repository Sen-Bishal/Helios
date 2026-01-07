// TODO LOTS of in development values

use bevy_ecs::prelude::*;
use glam::Vec3;
use crate::units::{Position3D, Distance};
#[derive(Component, Debug, Clone, Copy)]
pub struct Position(pub Position3D);

#[derive(Component, Debug, Clone, Copy)]
pub struct Velocity(pub Vec3);

impl Velocity {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self(Vec3::new(x, y, z))
    }
    pub fn zero() -> Self {
        Self(Vec3::ZERO)
    }
    pub fn speed(&self) -> f32 {
        self.0.length()
    }
}
#[derive(Component, Debug, Clone, Copy)]
pub struct Mass(pub f32);

impl Mass {
    pub fn from_grams(g: f32) -> Self {
        Self(g / 1000.0)
    }
    pub fn from_micrograms(ug: f32) -> Self {
        Self(ug / 1_000_000.0)
    }
}
#[derive(Component, Debug, Clone, Copy)]
pub struct ThermalState {
    pub temperature: f32,
    pub heat_energy: f32,
    pub heat_capacity: f32,
}

impl ThermalState {
    pub fn new(initial_temp: f32, heat_capacity: f32) ->  Self {
        Self {
            temperature: initial_temp,
            heat_energy: 0.0,
            heat_capacity,
        }
    }
    pub fn add_heat(&mut self, joules: f32) {
        self.heat_energy += joules;
        self.temperature += joules / self.heat_capacity;
    }
    pub const AMBIENT: f32 = 293.15; 
}
#[derive(Component, Debug, Clone, Copy)]
pub struct Acceleration(pub Vec3);

impl Acceleration {
    pub fn zero() -> Self {
        Self(Vec3::ZERO)
    }
    pub fn from_g_force(g: f32, direction: Vec3) -> Self {
        Self(direction.normalize() * g * 9.81)
    }
}
#[derive(Component, Debug, Clone, Copy)]
pub enum CollisionShape {
    Sphere { radius: Distance },
    Disk { radius: Distance, thickness: Distance },
    Ray { origin: Position3D, direction: Vec3, length: Distance },
}

#[derive(Component, Debug)]
pub enum EntityType {
    TinDroplet,
    Photon,
    PhotonPacket { count: u64 }, 
    Mirror,
    LaserBeam,
    WaferStage,
    ReticleStage,
    Debris,
}

#[derive(Component, Debug)]
pub struct Lifetime {
    pub remaining_seconds: f32,
}

impl Lifetime {
    pub fn new(seconds: f32) -> Self {
        Self {
            remaining_seconds: seconds,
        }
    }

    pub fn tick(&mut self, delta: f32) -> bool {
        self.remaining_seconds -= delta;
        self.remaining_seconds <= 0.0
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct OpticalMaterial {
    pub reflectivity: f32,
    pub absorption: f32,
}

impl OpticalMaterial {
    pub const BRAGG_MIRROR: Self = Self {
        reflectivity: 0.70,
        absorption: 0.30,
    };
    pub fn interact(&self, rng: &mut impl rand::Rng) -> bool {
        rng.gen::<f32>() < self.reflectivity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thermal_state() {
        let mut thermal = ThermalState::new(293.15, 1000.0);
        thermal.add_heat(1000.0); // Add 1kJ
        assert_eq!(thermal.temperature, 294.15); // Should increase by 1K
    }

    #[test]
    fn test_velocity_speed() {
        let vel = Velocity::new(3.0, 4.0, 0.0);
        assert_eq!(vel.speed(), 5.0); // 3-4-5 triangle
    }
}