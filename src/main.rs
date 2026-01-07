//! Architecture: 
//! - Hybrid precision: i128 for positions, f64 for physics
//! - ECS-based (bevy_ecs) for entity management
//! - Deterministic microsecond-level time stepping
//! - Modular subsystems: Source, Optics, Mechanics, Thermal

mod units;
mod components;
mod source;
mod interactions;
mod optics;
mod raytracing;
mod thermal;

use bevy_ecs::prelude::*;
use source::*;
use interactions::*;
use components::*;
use optics::*;
use raytracing::*;
use thermal::*;

#[derive(Debug)]
struct SimulationConfig {
    tick_duration: f32,
    total_duration: f32,
    burst_count: u32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            tick_duration: 1e-6, 
            total_duration: 0.002, 
            burst_count: 100,
        }
    }
}

fn main() {
    println!("╔═══════════════════════════════════════════════════════╗");
    println!("║           Helius - under Development                  ║");
    println!("║  Simulating High-NA photon dynamics at picometer      ║");
    println!("║                         PHASE 2                       ║");
    println!("╚═══════════════════════════════════════════════════════╝\n");
    let config = SimulationConfig::default();
    let mut world = World::new();
    let mut schedule = Schedule::default();

    world.insert_resource(SimulationTime::default());
    world.insert_resource(DropletGeneratorConfig::default());
    world.insert_resource(DropletGeneratorState::default());
    world.insert_resource(LaserTargetingSystem::default());
    world.insert_resource(OpticalSystemConfig::default());
    world.insert_resource(RayTracingStatistics::default());
    world.insert_resource(ThermalStatistics::default());

    schedule.add_systems((
        droplet_generator_system,
        laser_targeting_system,
        laser_droplet_interaction_system,
        physics_movement_system,
        plasma_to_debris_system,
        photon_mirror_interaction_system,
        photon_cleanup_system,
        thermal_dissipation_system,
        thermal_warning_system,
        lifetime_system,
        raytracing_statistics_system,
        thermal_statistics_system,
    ));

    println!("Configuration:");
    println!("  Tick duration: {:.2} μs", config.tick_duration * 1e6);
    println!("  Burst mode: {} droplets", config.burst_count);
    println!("  Total duration: {:.2} ms", config.total_duration * 1e3);
    println!("  Expected ticks: {}\n", 
        (config.total_duration / config.tick_duration) as u32);
    let start_time = std::time::Instant::now();
    let mut tick_count = 0u64;
    let mut last_report = 0u64;
    
    while world.resource::<SimulationTime>().total_seconds < config.total_duration as f64 {
        world.resource_mut::<SimulationTime>().tick(config.tick_duration);
        schedule.run(&mut world);
        
        tick_count += 1;
        if tick_count - last_report >= 100_000 {
            last_report = tick_count;
            let sim_time = world.resource::<SimulationTime>().total_seconds;
            let droplet_count = world.resource::<DropletGeneratorState>().droplet_count;
            
            println!("Tick {}: t={:.3}ms, Droplets spawned: {}", 
                tick_count, 
                sim_time * 1e3,
                droplet_count
            );
        }
    }

    let elapsed = start_time.elapsed();
    
    // Final statistics
    println!("\n╔═══════════════════════════════════════════════════════╗");
    println!("  ║                       Compiled                        ║");
    println!("  ╚═══════════════════════════════════════════════════════╝");
    
    let droplet_count = world.resource::<DropletGeneratorState>().droplet_count;
    let sim_time = world.resource::<SimulationTime>().total_seconds;
    
    println!("\nStatistics:");
    println!("  Total ticks: {}", tick_count);
    println!("  Simulated time: {:.3} ms", sim_time * 1e3);
    println!("  Droplets generated: {}", droplet_count);
    println!("  Wall clock time: {:.2} s", elapsed.as_secs_f64());
    println!("  Performance: {:.2}x realtime", 
        sim_time / elapsed.as_secs_f64());
    println!("  Ticks per second: {:.2}M", 
        tick_count as f64 / elapsed.as_secs_f64() / 1e6);

    let mut entity_counts = std::collections::HashMap::new();
    for entity in world.iter_entities() {
        if let Some(entity_type) = entity.get::<EntityType>() {
            let type_name = format!("{:?}", entity_type);
            *entity_counts.entry(type_name).or_insert(0) += 1;
        }
    }

    println!("\nFinal entity counts:");
    for (entity_type, count) in entity_counts.iter() {
        println!("  {}: {}", entity_type, count);
    }

    let ray_stats = world.resource::<RayTracingStatistics>();
    let thermal_stats = world.resource::<ThermalStatistics>();

    println!("\nOptical Statistics:");
    println!("  Total reflections: {}", ray_stats.total_reflections);
    println!("  Total absorptions: {}", ray_stats.total_absorptions);
    println!("  Active photon packets: {}", ray_stats.active_photon_packets);
    println!("  Average bounces per packet: {:.2}", ray_stats.average_bounces);

    println!("\nThermal Statistics:");
    println!("  Max temperature: {:.1}K", thermal_stats.max_temperature);
    println!("  Avg temperature: {:.1}K", thermal_stats.avg_temperature);
    println!("  Total heat energy: {:.2}J", thermal_stats.total_heat_energy);

    println!("\n✓ Simulation completed successfully");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_runs() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        world.insert_resource(DropletGeneratorConfig::default());
        world.insert_resource(DropletGeneratorState::default());

        let mut schedule = Schedule::default();
        schedule.add_systems(droplet_generator_system);
        for _ in 0..100 {
            world.resource_mut::<SimulationTime>().tick(1e-6);
            schedule.run(&mut world);
        }
        let state = world.resource::<DropletGeneratorState>();
        assert!(state.droplet_count > 0);
    }
}