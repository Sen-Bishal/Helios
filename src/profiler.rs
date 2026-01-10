use std::time::{Duration, Instant};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SystemTiming {
    pub name: String,
    pub call_count: u64,
    pub total_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
}

impl SystemTiming {
    pub fn new(name: String) -> Self {
        Self {
            name,
            call_count: 0,
            total_duration: Duration::ZERO,
            min_duration: Duration::MAX,
            max_duration: Duration::ZERO,
        }
    }

    pub fn record(&mut self, duration: Duration) {
        self.call_count += 1;
        self.total_duration += duration;
        self.min_duration = self.min_duration.min(duration);
        self.max_duration = self.max_duration.max(duration);
    }

    pub fn avg_duration(&self) -> Duration {
        if self.call_count > 0 {
            self.total_duration / self.call_count as u32
        } else {
            Duration::ZERO
        }
    }

    pub fn avg_micros(&self) -> f64 {
        self.avg_duration().as_secs_f64() * 1_000_000.0
    }

    pub fn total_micros(&self) -> f64 {
        self.total_duration.as_secs_f64() * 1_000_000.0
    }

    pub fn percentage(&self, total: Duration) -> f64 {
        if total.as_nanos() > 0 {
            (self.total_duration.as_nanos() as f64 / total.as_nanos() as f64) * 100.0
        } else {
            0.0
        }
    }
}

pub struct Profiler {
    timings: HashMap<String, SystemTiming>,
    frame_start: Instant,
    total_frames: u64,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            timings: HashMap::new(),
            frame_start: Instant::now(),
            total_frames: 0,
        }
    }

    pub fn start_frame(&mut self) {
        self.frame_start = Instant::now();
        self.total_frames += 1;
    }

    pub fn record_system(&mut self, name: &str, duration: Duration) {
        self.timings
            .entry(name.to_string())
            .or_insert_with(|| SystemTiming::new(name.to_string()))
            .record(duration);
    }

    pub fn report(&self) -> ProfileReport {
        let total_time: Duration = self.timings.values()
            .map(|t| t.total_duration)
            .sum();

        let mut systems: Vec<SystemTiming> = self.timings.values().cloned().collect();
        systems.sort_by(|a, b| b.total_duration.cmp(&a.total_duration));

        ProfileReport {
            total_frames: self.total_frames,
            total_time,
            systems,
        }
    }

    pub fn reset(&mut self) {
        self.timings.clear();
        self.total_frames = 0;
    }
}

pub struct ProfileReport {
    pub total_frames: u64,
    pub total_time: Duration,
    pub systems: Vec<SystemTiming>,
}

impl ProfileReport {
    pub fn print(&self) {
        println!("\n{}", "═".repeat(120));
        println!("PERFORMANCE PROFILE");
        println!("{}", "═".repeat(120));
        
        println!("\nOverall:");
        println!("  Total frames: {}", self.total_frames);
        println!("  Total time: {:.2} ms", self.total_time.as_secs_f64() * 1000.0);
        println!("  Avg frame time: {:.2} μs", 
            self.total_time.as_secs_f64() * 1_000_000.0 / self.total_frames as f64);

        println!("\n{:<40} {:<12} {:<12} {:<12} {:<12} {:<10}", 
            "System", "Calls", "Total(μs)", "Avg(μs)", "Max(μs)", "% Time");
        println!("{}", "─".repeat(120));

        for timing in &self.systems {
            println!("{:<40} {:<12} {:<12.2} {:<12.2} {:<12.2} {:<10.2}", 
                timing.name,
                timing.call_count,
                timing.total_micros(),
                timing.avg_micros(),
                timing.max_duration.as_secs_f64() * 1_000_000.0,
                timing.percentage(self.total_time)
            );
        }

        println!("\nBottlenecks (>10% of total time):");
        for timing in self.systems.iter().filter(|t| t.percentage(self.total_time) > 10.0) {
            println!("  ⚠️  {} - {:.1}% ({:.2}ms total)", 
                timing.name,
                timing.percentage(self.total_time),
                timing.total_micros() / 1000.0
            );
        }
    }
}

pub struct ScopedTimer<'a> {
    profiler: &'a mut Profiler,
    system_name: String,
    start: Instant,
}

impl<'a> ScopedTimer<'a> {
    pub fn new(profiler: &'a mut Profiler, system_name: &str) -> Self {
        Self {
            profiler,
            system_name: system_name.to_string(),
            start: Instant::now(),
        }
    }
}

impl<'a> Drop for ScopedTimer<'a> {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        self.profiler.record_system(&self.system_name, duration);
    }
}