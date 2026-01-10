use bevy_ecs::prelude::*;
use glam::Vec3;
use rand::Rng;
use crate::units::Position3D;
use crate::components::*;
use crate::optics::MirrorSurface;

#[derive(Component, Debug)]
pub struct PhotonPacket {
    pub photon_count: u64,
    pub wavelength: f32,
    pub energy_per_photon: f32,
    pub bounces: u32,
}

impl PhotonPacket {
    pub const EUV_WAVELENGTH: f32 = 13.5e-9;
    pub const MAX_BOUNCES: u32 = 15;

    pub fn new(photon_count: u64) -> Self {
        const PLANCK: f64 = 6.626e-34;
        const LIGHT_SPEED: f64 = 3e8;
        let energy = (PLANCK * LIGHT_SPEED / Self::EUV_WAVELENGTH as f64) as f32;

        Self {
            photon_count,
            wavelength: Self::EUV_WAVELENGTH,
            energy_per_photon: energy,
            bounces: 0,
        }
    }

    pub fn total_energy(&self) -> f32 {
        self.photon_count as f32 * self.energy_per_photon
    }
}

pub fn photon_mirror_interaction_system(
    mut commands: Commands,
    mut photons: Query<(Entity, &mut Position, &mut Velocity, &mut PhotonPacket)>,
    mut mirrors: Query<(&Position, &MirrorSurface, &OpticalMaterial, &mut ThermalState), Without<PhotonPacket>>,
    mut stats: ResMut<RayTracingStatistics>,
) {
    if mirrors.is_empty() {
        return;
    }

    let (_, mirror_surface, mirror_material, _) = mirrors.single();
    
    let (mirror_center, mirror_radius) = match &mirror_surface.geometry {
        crate::optics::SurfaceGeometry::Spherical { radius, center } => {
            (*center, radius.as_meters_f64() as f32)
        }
        _ => return,
    };

    let mut absorbed_heat = 0.0f32;
    let mut to_despawn = Vec::new();
    let mirror_center_vec = mirror_center.to_vec3();

    for (photon_entity, mut photon_pos, mut photon_vel, mut packet) in photons.iter_mut() {
        if packet.bounces >= PhotonPacket::MAX_BOUNCES {
            to_despawn.push(photon_entity);
            continue;
        }

        let photon_vec = photon_pos.0.to_vec3();
        let distance = photon_vec.distance(mirror_center_vec);
        
        if distance <= mirror_radius {
            let mut rng = rand::thread_rng();
            let reflects = rng.gen::<f32>() < mirror_material.reflectivity;

            if reflects {
                let ray_direction = photon_vel.0.normalize();
                let normal = mirror_surface.geometry.normal_at(photon_pos.0);
                let reflected = ray_direction - 2.0 * ray_direction.dot(normal) * normal;
                
                photon_vel.0 = reflected.normalize() * photon_vel.0.length();
                packet.bounces += 1;
                stats.total_reflections += 1;
            } else {
                absorbed_heat += packet.total_energy() * mirror_material.absorption;
                stats.total_absorptions += 1;
                to_despawn.push(photon_entity);
            }
        }
    }

    if absorbed_heat > 0.0 {
        let (_, _, _, mut thermal) = mirrors.single_mut();
        thermal.add_heat(absorbed_heat);
    }

    for entity in to_despawn {
        commands.entity(entity).despawn();
    }
}

pub fn photon_cleanup_system(
    mut commands: Commands,
    query: Query<(Entity, &Position, &PhotonPacket)>,
) {
    const MAX_DISTANCE: f32 = 20.0;

    for (entity, pos, packet) in query.iter() {
        let distance = pos.0.to_vec3().length();
        
        if distance > MAX_DISTANCE || packet.bounces >= PhotonPacket::MAX_BOUNCES {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Resource, Default, Debug)]
pub struct RayTracingStatistics {
    pub total_reflections: u64,
    pub total_absorptions: u64,
    pub active_photon_packets: u32,
    pub average_bounces: f32,
}

pub fn raytracing_statistics_system(
    photons: Query<&PhotonPacket>,
    mut stats: ResMut<RayTracingStatistics>,
) {
    stats.active_photon_packets = photons.iter().count() as u32;
    
    if stats.active_photon_packets > 0 {
        let total_bounces: u32 = photons.iter().map(|p| p.bounces).sum();
        stats.average_bounces = total_bounces as f32 / stats.active_photon_packets as f32;
    }
}