use bevy_ecs::prelude::*;
use crate::components::ThermalState;
use crate::source::SimulationTime;

#[derive(Component, Debug, Clone)]
pub struct CoolingSystem {
    pub cooling_power: f32,
    pub target_temperature: f32,
    pub efficiency: f32,
}

impl CoolingSystem {
    pub fn active_cooling(cooling_power_watts: f32) -> Self {
        Self {
            cooling_power: cooling_power_watts,
            target_temperature: ThermalState::AMBIENT,
            efficiency: 0.95,
        }
    }

    pub fn passive_cooling() -> Self {
        Self {
            cooling_power: 100.0,
            target_temperature: ThermalState::AMBIENT,
            efficiency: 0.5,
        }
    }
}

impl Default for CoolingSystem {
    fn default() -> Self {
        Self::active_cooling(1000.0)
    }
}

pub fn thermal_dissipation_system(
    time: Res<SimulationTime>,
    mut query: Query<(&mut ThermalState, Option<&CoolingSystem>)>,
) {
    for (mut thermal, cooling_opt) in query.iter_mut() {
        let delta_temp = thermal.temperature - ThermalState::AMBIENT;
        
        if delta_temp > 0.0 {
            let cooling = if let Some(cooling_system) = cooling_opt {
                let max_heat_removal = cooling_system.cooling_power * time.delta_seconds;
                let proportional_cooling = delta_temp * cooling_system.efficiency * 0.1;
                max_heat_removal.min(proportional_cooling * thermal.heat_capacity)
            } else {
                let natural_convection = delta_temp * 0.01 * thermal.heat_capacity * time.delta_seconds;
                natural_convection
            };

            thermal.heat_energy -= cooling;
            thermal.temperature = ThermalState::AMBIENT + 
                (thermal.heat_energy / thermal.heat_capacity).max(0.0);
        }
    }
}

pub fn thermal_warning_system(
    query: Query<(&ThermalState, Entity)>,
) {
    const WARNING_TEMP: f32 = 400.0;
    const CRITICAL_TEMP: f32 = 600.0;

    for (thermal, entity) in query.iter() {
        if thermal.temperature > CRITICAL_TEMP {
            eprintln!("CRITICAL: Entity {:?} temperature {:.1}K exceeds safe limit!", 
                entity, thermal.temperature);
        } else if thermal.temperature > WARNING_TEMP {
            eprintln!("WARNING: Entity {:?} temperature {:.1}K elevated", 
                entity, thermal.temperature);
        }
    }
}

#[derive(Resource, Default, Debug)]
pub struct ThermalStatistics {
    pub max_temperature: f32,
    pub avg_temperature: f32,
    pub total_heat_energy: f32,
}

pub fn thermal_statistics_system(
    query: Query<&ThermalState>,
    mut stats: ResMut<ThermalStatistics>,
) {
    let mut max_temp = 0.0f32;
    let mut total_temp = 0.0f32;
    let mut total_energy = 0.0f32;
    let mut count = 0;

    for thermal in query.iter() {
        max_temp = max_temp.max(thermal.temperature);
        total_temp += thermal.temperature;
        total_energy += thermal.heat_energy;
        count += 1;
    }

    if count > 0 {
        stats.max_temperature = max_temp;
        stats.avg_temperature = total_temp / count as f32;
        stats.total_heat_energy = total_energy;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cooling_system() {
        let cooling = CoolingSystem::active_cooling(1000.0);
        assert_eq!(cooling.cooling_power, 1000.0);
        assert!(cooling.efficiency > 0.0);
    }

    #[test]
    fn test_thermal_dissipation() {
        let mut thermal = ThermalState::new(400.0, 1000.0);
        thermal.heat_energy = 107_000.0;
        
        let delta_temp = thermal.temperature - ThermalState::AMBIENT;
        assert!(delta_temp > 100.0);
    }
}