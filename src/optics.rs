//! Optical subsystem: Mirrors, reflectors, and light transport

use bevy_ecs::prelude::*;
use glam::{Vec3, Quat};
use crate::units::{Position3D, Distance};
use crate::components::*;
use crate::thermal::CoolingSystem;

#[derive(Component, Debug, Clone)]
pub struct MirrorSurface {
    pub geometry: SurfaceGeometry,
    pub orientation: Quat,
    pub radius: Distance,
}

#[derive(Debug, Clone)]
pub enum SurfaceGeometry {
    Ellipsoid {
        semi_axes: Vec3,
        focus1: Position3D,
        focus2: Position3D,
    },
    Spherical {
        radius: Distance,
        center: Position3D,
    },
    Planar {
        normal: Vec3,
    },
}

impl SurfaceGeometry {
    pub fn normal_at(&self, point: Position3D) -> Vec3 {
        match self {
            SurfaceGeometry::Ellipsoid { semi_axes, focus1, .. } => {
                let center = focus1.to_vec3();
                let p = point.to_vec3() - center;
                
                let normal = Vec3::new(
                    2.0 * p.x / (semi_axes.x * semi_axes.x),
                    2.0 * p.y / (semi_axes.y * semi_axes.y),
                    2.0 * p.z / (semi_axes.z * semi_axes.z),
                );
                normal.normalize()
            }
            SurfaceGeometry::Spherical { center, .. } => {
                let p = point.to_vec3();
                let c = center.to_vec3();
                (p - c).normalize()
            }
            SurfaceGeometry::Planar { normal } => *normal,
        }
    }

    pub fn ray_intersection(
        &self,
        ray_origin: Position3D,
        ray_direction: Vec3,
    ) -> (bool, Option<Position3D>, f32) {
        match self {
            SurfaceGeometry::Spherical { radius, center } => {
                Self::ray_sphere_intersection(ray_origin, ray_direction, *center, *radius)
            }
            SurfaceGeometry::Planar { normal } => {
                Self::ray_plane_intersection(ray_origin, ray_direction, *normal)
            }
            SurfaceGeometry::Ellipsoid { semi_axes, focus1, .. } => {
                let max_axis = semi_axes.x.max(semi_axes.y).max(semi_axes.z);
                let approx_radius = Distance::from_meters_f64(max_axis as f64);
                Self::ray_sphere_intersection(ray_origin, ray_direction, *focus1, approx_radius)
            }
        }
    }

    fn ray_sphere_intersection(
        ray_origin: Position3D,
        ray_direction: Vec3,
        sphere_center: Position3D,
        sphere_radius: Distance,
    ) -> (bool, Option<Position3D>, f32) {
        let o = ray_origin.to_vec3();
        let d = ray_direction.normalize();
        let c = sphere_center.to_vec3();
        let r = sphere_radius.as_meters_f64() as f32;

        let oc = o - c;
        let a = d.dot(d);
        let b = 2.0 * oc.dot(d);
        let c_term = oc.dot(oc) - r * r;
        
        let discriminant = b * b - 4.0 * a * c_term;
        
        if discriminant < 0.0 {
            return (false, None, f32::INFINITY);
        }

        let t = (-b - discriminant.sqrt()) / (2.0 * a);
        
        if t < 0.0 {
            return (false, None, f32::INFINITY);
        }

        let hit_point = o + d * t;
        (true, Some(Position3D::from_vec3(hit_point)), t)
    }

    fn ray_plane_intersection(
        ray_origin: Position3D,
        ray_direction: Vec3,
        plane_normal: Vec3,
    ) -> (bool, Option<Position3D>, f32) {
        let d = ray_direction.normalize();
        let n = plane_normal.normalize();
        
        let denom = n.dot(d);
        
        if denom.abs() < 1e-6 {
            return (false, None, f32::INFINITY);
        }

        let o = ray_origin.to_vec3();
        let t = -(n.dot(o)) / denom;
        
        if t < 0.0 {
            return (false, None, f32::INFINITY);
        }

        let hit_point = o + d * t;
        (true, Some(Position3D::from_vec3(hit_point)), t)
    }
}

#[derive(Resource, Debug, Clone)]
pub struct OpticalSystemConfig {
    pub collector_mirror: CollectorMirrorSpec,
    pub projection_mirrors: Vec<MirrorSpec>,
}

#[derive(Debug, Clone)]
pub struct CollectorMirrorSpec {
    pub position: Position3D,
    pub semi_major_axis: f32,
    pub semi_minor_axis: f32,
    pub focal_length: f32,
}

#[derive(Debug, Clone)]
pub struct MirrorSpec {
    pub id: u32,
    pub position: Position3D,
    pub radius: Distance,
    pub curvature_radius: Distance,
}

impl Default for OpticalSystemConfig {
    fn default() -> Self {
        Self {
            collector_mirror: CollectorMirrorSpec {
                position: Position3D::zero(),
                semi_major_axis: 0.3,
                semi_minor_axis: 0.2,
                focal_length: 0.5,
            },
            projection_mirrors: vec![
                MirrorSpec {
                    id: 1,
                    position: Position3D::new(
                        Distance::from_meters(1),
                        Distance::ZERO,
                        Distance::ZERO,
                    ),
                    radius: Distance::from_millimeters(200),
                    curvature_radius: Distance::from_meters(2),
                },
                MirrorSpec {
                    id: 2,
                    position: Position3D::new(
                        Distance::from_meters(2),
                        Distance::from_millimeters(500),
                        Distance::ZERO,
                    ),
                    radius: Distance::from_millimeters(150),
                    curvature_radius: Distance::from_meters(3),
                },
            ],
        }
    }
}

pub fn spawn_optical_system(
    mut commands: Commands,
    config: Res<OpticalSystemConfig>,
) {
    let collector = &config.collector_mirror;
    
    commands.spawn((
        Position(collector.position),
        MirrorSurface {
            geometry: SurfaceGeometry::Ellipsoid {
                semi_axes: Vec3::new(
                    collector.semi_major_axis,
                    collector.semi_minor_axis,
                    collector.semi_minor_axis,
                ),
                focus1: Position3D::zero(),
                focus2: Position3D::new(
                    Distance::from_meters_f64(collector.focal_length as f64),
                    Distance::ZERO,
                    Distance::ZERO,
                ),
            },
            orientation: Quat::IDENTITY,
            radius: Distance::from_meters_f64(collector.semi_major_axis as f64),
        },
        OpticalMaterial::BRAGG_MIRROR,
        ThermalState::new(ThermalState::AMBIENT, 5000.0),
        EntityType::Mirror,
    ));

    for mirror_spec in config.projection_mirrors.iter() {
        commands.spawn((
            Position(mirror_spec.position),
            MirrorSurface {
                geometry: SurfaceGeometry::Spherical {
                    radius: mirror_spec.curvature_radius,
                    center: mirror_spec.position,
                },
                orientation: Quat::IDENTITY,
                radius: mirror_spec.radius,
            },
            OpticalMaterial::BRAGG_MIRROR,
            ThermalState::new(ThermalState::AMBIENT, 2000.0),
            EntityType::Mirror,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_normal() {
        let center = Position3D::zero();
        let surface = SurfaceGeometry::Spherical {
            radius: Distance::from_meters(1),
            center,
        };
        
        let point = Position3D::new(
            Distance::from_meters(1),
            Distance::ZERO,
            Distance::ZERO,
        );
        
        let normal = surface.normal_at(point);
        assert!((normal.x - 1.0).abs() < 1e-6);
        assert!(normal.y.abs() < 1e-6);
    }

    #[test]
    fn test_ray_sphere_hit() {
        let origin = Position3D::new(
            Distance::from_meters(-2),
            Distance::ZERO,
            Distance::ZERO,
        );
        let direction = Vec3::X;
        let center = Position3D::zero();
        let radius = Distance::from_meters(1);

        let (hit, point, _) = SurfaceGeometry::ray_sphere_intersection(
            origin, direction, center, radius
        );

        assert!(hit);
        assert!(point.is_some());
    }
}