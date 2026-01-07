//TODO Lots to clean up in optimization before phase 04
use bevy_ecs::prelude::*;
use glam::Vec3;
use crate::units::{Position3D, Distance};
use crate::components::*;
use crate::source::{DropletState, LaserBeam, SimulationTime};

pub fn laser_droplet_interaction_system(
    mut commands: Commands,
    mut droplets: Query<(Entity, &Position, &mut DropletState, &mut CollisionShape), With<EntityType>>,
    mut lasers: Query<(Entity, &Position, &mut LaserBeam)>,
) {
    for (laser_entity, laser_pos, mut laser) in lasers.iter_mut() {
        if laser.has_fired {
            continue;
        }

        for (droplet_entity, droplet_pos, mut state, mut shape) in droplets.iter_mut() {
            // Check if laser hits droplet based on current shape
            let hit = match *shape {
                CollisionShape::Sphere { radius } => {
                    ray_sphere_intersection(laser_pos.0, droplet_pos.0, radius)
                }
                CollisionShape::Disk { radius, thickness } => {
                    ray_disk_intersection(laser_pos.0, droplet_pos.0, radius, thickness)
                }
                _ => false,
            };

            if hit {
                laser.has_fired = true;
                match (*state, laser.is_prepulse) {
                    (DropletState::Spherical, true) => {
                        *state = DropletState::Pancaked;
                        if let CollisionShape::Sphere { radius } = *shape {
                            *shape = CollisionShape::Disk {
                                radius: radius * 2, // Flatten increases surface area
                                thickness: radius / 4, // Much thinner
                            };
                        }
                    }
                    (DropletState::Pancaked, false) => {
                        *state = DropletState::Plasma;
                        
                        // Spawn photon packets from the plasma
                        spawn_photon_packets(
                            &mut commands,
                            droplet_pos.0,
                            laser.power,
                        );

                        // Schedule droplet for debris conversion after plasma lifetime
                        commands.entity(droplet_entity).insert(Lifetime::new(0.000_01));
                    }

                    _ => {} // Invalid state transitions are ignored
                }

                break; // Laser can only hit one droplet
            }
        }
    }
}

/// Ray-sphere intersection test
/// Returns true if ray from laser_pos intersects sphere at droplet_pos with given radius
fn ray_sphere_intersection(
    laser_pos: Position3D,
    droplet_pos: Position3D,
    radius: Distance,
) -> bool {
    // Convert to Vec3 for math
    let laser = laser_pos.to_vec3();
    let droplet = droplet_pos.to_vec3();
    let r = radius.as_meters_f64() as f32;

    // Simple distance check (assuming laser is point source at exact position)
    let distance = laser.distance(droplet);
    distance <= r
}

/// Ray-disk intersection test
/// Disk is assumed perpendicular to the X-axis (travel direction)
fn ray_disk_intersection(
    laser_pos: Position3D,
    droplet_pos: Position3D,
    radius: Distance,
    thickness: Distance,
) -> bool {
    let laser = laser_pos.to_vec3();
    let droplet = droplet_pos.to_vec3();
    let r = radius.as_meters_f64() as f32;
    let t = thickness.as_meters_f64() as f32;

    // Check if laser is within thickness along X
    let x_diff = (laser.x - droplet.x).abs();
    if x_diff > t / 2.0 {
        return false;
    }

    // Check if laser is within radius in YZ plane
    let yz_distance = ((laser.y - droplet.y).powi(2) + (laser.z - droplet.z).powi(2)).sqrt();
    yz_distance <= r
}

/// Spawns photon packets representing EUV light emission from plasma
fn spawn_photon_packets(
    commands: &mut Commands,
    plasma_position: Position3D,
    laser_power: f32,
) {
    const PACKET_COUNT: u32 = 1000;
    const EUV_WAVELENGTH: f32 = 13.5e-9;

    const PLANCK: f64 = 6.626e-34;
    const LIGHT_SPEED: f64 = 3e8;
    let photon_energy = (PLANCK * LIGHT_SPEED / EUV_WAVELENGTH as f64) as f32;

    let euv_power = laser_power * 0.02;
    let total_photons = (euv_power / photon_energy) as u64;

    use rand::Rng;
    let mut rng = rand::thread_rng();

    for _ in 0..PACKET_COUNT {
        let theta = rng.gen_range(0.0..std::f32::consts::TAU);
        let phi = rng.gen_range(0.0..std::f32::consts::PI);
        
        let direction = Vec3::new(
            phi.sin() * theta.cos(),
            phi.sin() * theta.sin(),
            phi.cos(),
        );

        let velocity = direction * 3e8;

        commands.spawn((
            Position(plasma_position),
            Velocity(velocity),
            crate::raytracing::PhotonPacket::new(total_photons / PACKET_COUNT as u64),
            Lifetime::new(0.000_1),
        ));
    }
}

/// System that converts plasma back to debris after lifetime expires
pub fn plasma_to_debris_system(
    mut query: Query<(&mut DropletState, &Lifetime)>,
) {
    for (mut state, lifetime) in query.iter_mut() {
        if *state == DropletState::Plasma && lifetime.remaining_seconds <= 0.0 {
            *state = DropletState::Debris;
        }
    }
}

/// System that moves all entities based on velocity
pub fn physics_movement_system(
    time: Res<SimulationTime>,
    mut query: Query<(&mut Position, &Velocity)>,
) {
    for (mut pos, vel) in query.iter_mut() {
        // Convert velocity (m/s) to displacement over delta time
        let displacement = vel.0 * time.delta_seconds;
        
        // Update position (convert Vec3 back to Position3D with precision)
        let new_pos_vec = pos.0.to_vec3() + displacement;
        pos.0 = Position3D::from_vec3(new_pos_vec);
    }
}

/// System that cleans up entities whose lifetime has expired
pub fn lifetime_system(
    mut commands: Commands,
    time: Res<SimulationTime>,
    mut query: Query<(Entity, &mut Lifetime)>,
) {
    for (entity, mut lifetime) in query.iter_mut() {
        if lifetime.tick(time.delta_seconds) {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ray_sphere_intersection() {
        let laser_pos = Position3D::new(
            Distance::ZERO,
            Distance::ZERO,
            Distance::ZERO,
        );
        let droplet_pos = Position3D::new(
            Distance::from_micrometers(20),
            Distance::ZERO,
            Distance::ZERO,
        );
        let radius = Distance::from_micrometers(30);

        assert!(ray_sphere_intersection(laser_pos, droplet_pos, radius));
    }

    #[test]
    fn test_ray_sphere_miss() {
        let laser_pos = Position3D::zero();
        let droplet_pos = Position3D::new(
            Distance::from_millimeters(1),
            Distance::ZERO,
            Distance::ZERO,
        );
        let radius = Distance::from_micrometers(30);

        assert!(!ray_sphere_intersection(laser_pos, droplet_pos, radius));
    }
}