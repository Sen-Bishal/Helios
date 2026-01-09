mod units;
mod components;
mod source;
mod interactions;
mod optics;
mod raytracing;
mod thermal;

use bevy_ecs::prelude::*;
use units::*;
use components::*;
use source::*;
use interactions::*;
use optics::*;
use raytracing::*;
use thermal::*;

fn main() {
    println!("╔═══════════════════════════════════════════════════════╗");
    println!("║  LITHOS - EUV Lithography Simulator                  ║");
    println!("║  High-Performance Terminal Mode                      ║");
    println!("╚═══════════════════════════════════════════════════════╝\n");

    let mut world = World::new();
    
    world.insert_resource(SimulationTime::default());
    world.insert_resource(DropletGeneratorConfig::default());
    world.insert_resource(DropletGeneratorState::default());
    world.insert_resource(LaserTargetingSystem::default());
    world.insert_resource(RayTracingStatistics::default());
    world.insert_resource(ThermalStatistics::default());

    world.spawn((
        Position(Position3D::zero()),
        MirrorSurface {
            geometry: SurfaceGeometry::Spherical {
                radius: Distance::from_meters(5),
                center: Position3D::zero(),
            },
            orientation: glam::Quat::IDENTITY,
            radius: Distance::from_meters(5),
        },
        OpticalMaterial::BRAGG_MIRROR,
        ThermalState::new(293.15, 5000.0),
    ));

    let mirror_count = world.query::<&MirrorSurface>().iter(&world).count();
    println!("System Initialization:");
    println!("  └─ Mirrors spawned: {}", mirror_count);
    println!("  └─ Droplet frequency: 50 kHz");
    println!("  └─ Simulation tick: 1 μs");
    println!("  └─ Mirror radius: 5.0 m");
    println!();

    let mut schedule = Schedule::default();
    schedule.add_systems((
        droplet_generator_system,
        laser_targeting_system,
        physics_movement_system,
        laser_droplet_interaction_system,
        plasma_to_debris_system,
        photon_mirror_interaction_system,
        photon_cleanup_system,
        thermal_dissipation_system,
        raytracing_statistics_system,
        thermal_statistics_system,
        lifetime_system,
    ));

    let start_time = std::time::Instant::now();
    let mut tick_count = 0u64;
    let max_ticks = 50000;
    let report_interval = 5000;

    println!("Starting simulation... (50ms total, reporting every 5ms)\n");
    println!("{:<8} {:<12} {:<10} {:<10} {:<12} {:<12} {:<10} {:<10}", 
        "Tick", "Time(μs)", "Droplets", "Photons", "Reflections", "Absorptions", "MaxTemp", "AvgBounce");
    println!("{}", "─".repeat(100));

    while tick_count < max_ticks {
        world.resource_mut::<SimulationTime>().tick(1e-6);
        schedule.run(&mut world);
        tick_count += 1;

        if tick_count % report_interval == 0 {
            let droplet_count = world.resource::<DropletGeneratorState>().droplet_count;
            let total_reflections = world.resource::<RayTracingStatistics>().total_reflections;
            let total_absorptions = world.resource::<RayTracingStatistics>().total_absorptions;
            let average_bounces = world.resource::<RayTracingStatistics>().average_bounces;
            let max_temperature = world.resource::<ThermalStatistics>().max_temperature;
            let sim_time = world.resource::<SimulationTime>().total_seconds * 1e6;
            
            let photons = world.query::<&PhotonPacket>().iter(&world).count();
            
            println!("{:<8} {:<12.1} {:<10} {:<10} {:<12} {:<12} {:<10.2} {:<10.2}", 
                tick_count,
                sim_time,
                droplet_count,
                photons,
                total_reflections,
                total_absorptions,
                max_temperature,
                average_bounces
            );
        }
    }

    let elapsed = start_time.elapsed();
    let ray_stats = world.resource::<RayTracingStatistics>();
    let thermal_stats = world.resource::<ThermalStatistics>();
    let sim_time = world.resource::<SimulationTime>().total_seconds;

    println!("\n{}", "═".repeat(100));
    println!("SIMULATION COMPLETE");
    println!("{}", "═".repeat(100));
    
    println!("\n┌─ Performance Metrics");
    println!("│  ├─ Total ticks: {}", tick_count);
    println!("│  ├─ Simulated time: {:.3} ms", sim_time * 1e3);
    println!("│  ├─ Wall clock time: {:.3} s", elapsed.as_secs_f64());
    println!("│  ├─ Speed: {:.2}x realtime", sim_time / elapsed.as_secs_f64());
    println!("│  └─ Ticks/second: {:.2} M", tick_count as f64 / elapsed.as_secs_f64() / 1e6);

    println!("\n┌─ Source Statistics");
    let droplet_count = world.resource::<DropletGeneratorState>().droplet_count;
    println!("│  ├─ Droplets generated: {}", droplet_count);
    println!("│  ├─ Plasma events: ~{}", droplet_count / 2);
    println!("│  └─ Expected photon packets: ~{}", droplet_count * 500);

    println!("\n┌─ Optical Statistics");
    println!("│  ├─ Total reflections: {}", ray_stats.total_reflections);
    println!("│  ├─ Total absorptions: {}", ray_stats.total_absorptions);
    println!("│  ├─ Reflection ratio: {:.1}%", 
        ray_stats.total_reflections as f64 / (ray_stats.total_reflections + ray_stats.total_absorptions) as f64 * 100.0);
    println!("│  ├─ Active photon packets: {}", ray_stats.active_photon_packets);
    println!("│  └─ Average bounces/packet: {:.2}", ray_stats.average_bounces);

    println!("\n┌─ Thermal Statistics");
    println!("│  ├─ Max temperature: {:.2} K", thermal_stats.max_temperature);
    println!("│  ├─ Avg temperature: {:.2} K", thermal_stats.avg_temperature);
    println!("│  ├─ Temperature rise: {:.2} K", thermal_stats.max_temperature - 293.15);
    println!("│  └─ Total heat absorbed: {:.2} J", thermal_stats.total_heat_energy);

    let mut entity_counts = std::collections::HashMap::new();
    for entity in world.iter_entities() {
        if let Some(entity_type) = entity.get::<EntityType>() {
            let type_name = format!("{:?}", entity_type);
            *entity_counts.entry(type_name).or_insert(0) += 1;
        }
    }

    if !entity_counts.is_empty() {
        println!("\n┌─ Entity Distribution");
        for (entity_type, count) in entity_counts.iter() {
            println!("│  ├─ {}: {}", entity_type, count);
        }
    }

    println!("\n✓ Simulation completed successfully\n");
}