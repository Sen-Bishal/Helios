use eframe::egui;
use egui::{Color32, FontId, RichText, Stroke, Rounding};
use egui_plot::{Line, Plot, PlotPoints};
use std::collections::VecDeque;

pub struct LithosGui {
    pub simulation_state: SimulationState,
    pub stats_history: StatsHistory,
}

#[derive(Debug, Clone)]
pub struct SimulationState {
    pub is_running: bool,
    pub tick_count: u64,
    pub simulation_time_us: f64,
    pub droplet_count: u64,
    pub plasma_count: u32,
    pub active_photons: u32,
    pub total_reflections: u64,
    pub total_absorptions: u64,
    pub average_bounces: f32,
    pub max_temperature: f32,
    pub avg_temperature: f32,
    pub total_heat_energy: f32,
}

impl Default for SimulationState {
    fn default() -> Self {
        Self {
            is_running: false,
            tick_count: 0,
            simulation_time_us: 0.0,
            droplet_count: 0,
            plasma_count: 0,
            active_photons: 0,
            total_reflections: 0,
            total_absorptions: 0,
            average_bounces: 0.0,
            max_temperature: 293.15,
            avg_temperature: 293.15,
            total_heat_energy: 0.0,
        }
    }
}

#[derive(Debug)]
pub struct StatsHistory {
    pub time_points: VecDeque<f64>,
    pub photon_counts: VecDeque<f64>,
    pub max_temps: VecDeque<f64>,
    pub avg_temps: VecDeque<f64>,
    max_points: usize,
}

impl StatsHistory {
    pub fn new(capacity: usize) -> Self {
        Self {
            time_points: VecDeque::with_capacity(capacity),
            photon_counts: VecDeque::with_capacity(capacity),
            max_temps: VecDeque::with_capacity(capacity),
            avg_temps: VecDeque::with_capacity(capacity),
            max_points: capacity,
        }
    }

    pub fn push(&mut self, time_us: f64, photons: u32, max_temp: f32, avg_temp: f32) {
        if self.time_points.len() >= self.max_points {
            self.time_points.pop_front();
            self.photon_counts.pop_front();
            self.max_temps.pop_front();
            self.avg_temps.pop_front();
        }

        self.time_points.push_back(time_us);
        self.photon_counts.push_back(photons as f64);
        self.max_temps.push_back(max_temp as f64);
        self.avg_temps.push_back(avg_temp as f64);
    }
}

impl LithosGui {
    pub fn new() -> Self {
        Self {
            simulation_state: SimulationState::default(),
            stats_history: StatsHistory::new(1000),
        }
    }

    fn apply_apple_style(ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        
        style.visuals.window_fill = Color32::from_rgb(250, 250, 250);
        style.visuals.panel_fill = Color32::from_rgb(250, 250, 250);
        style.visuals.extreme_bg_color = Color32::from_rgb(255, 255, 255);
        
        style.visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(245, 245, 247);
        style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(240, 240, 243);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(235, 235, 238);
        style.visuals.widgets.active.bg_fill = Color32::from_rgb(0, 122, 255);
        
        style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(142, 142, 147));
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(60, 60, 67));
        style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::from_rgb(0, 122, 255));
        style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
        
        style.visuals.window_rounding = Rounding::same(12.0);
        style.visuals.widgets.noninteractive.rounding = Rounding::same(8.0);
        style.visuals.widgets.inactive.rounding = Rounding::same(8.0);
        style.visuals.widgets.hovered.rounding = Rounding::same(8.0);
        style.visuals.widgets.active.rounding = Rounding::same(8.0);
        
        style.spacing.item_spacing = egui::vec2(12.0, 8.0);
        style.spacing.button_padding = egui::vec2(16.0, 8.0);
        style.spacing.window_margin = egui::Margin::same(16.0);
        
        ctx.set_style(style);
    }
}

impl eframe::App for LithosGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        Self::apply_apple_style(ctx);
        
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.heading(RichText::new("LITHOS").size(24.0).color(Color32::from_rgb(60, 60, 67)));
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let play_pause_text = if self.simulation_state.is_running { "⏸" } else { "▶" };
                    if ui.button(RichText::new(play_pause_text).size(20.0)).clicked() {
                        self.simulation_state.is_running = !self.simulation_state.is_running;
                    }
                    
                    if ui.button(RichText::new("⏭").size(20.0)).clicked() {
                    }
                    
                    ui.button(RichText::new("⚙").size(20.0));
                });
            });
            ui.add_space(8.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(16.0);
            
            let available_height = ui.available_height();
            let viz_height = available_height * 0.5;
            
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), viz_height),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(245, 245, 247))
                        .rounding(Rounding::same(12.0))
                        .inner_margin(egui::Margin::same(24.0))
                        .show(ui, |ui| {
                            self.render_system_diagram(ui);
                        });
                }
            );

            ui.add_space(24.0);
            
            ui.columns(3, |columns| {
                self.render_source_panel(&mut columns[0]);
                self.render_optics_panel(&mut columns[1]);
                self.render_thermal_panel(&mut columns[2]);
            });

            ui.add_space(16.0);
            self.render_charts(ui);
        });

        ctx.request_repaint();
    }
}

impl LithosGui {
    fn render_system_diagram(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            
            ui.label(
                RichText::new("System Architecture")
                    .size(18.0)
                    .color(Color32::from_rgb(60, 60, 67))
            );
            
            ui.add_space(32.0);
            
            ui.horizontal(|ui| {
                ui.add_space(40.0);
                
                self.render_stage_box(ui, "SOURCE", vec![
                    format!("{} droplets", self.simulation_state.droplet_count),
                    format!("{} plasma events", self.simulation_state.plasma_count),
                ], Color32::from_rgb(88, 86, 214));
                
                ui.add_space(20.0);
                ui.label(RichText::new("→").size(32.0).color(Color32::from_rgb(142, 142, 147)));
                ui.add_space(20.0);
                
                self.render_stage_box(ui, "OPTICS", vec![
                    format!("{} photons", self.simulation_state.active_photons),
                    format!("{:.1} avg bounces", self.simulation_state.average_bounces),
                ], Color32::from_rgb(0, 122, 255));
                
                ui.add_space(20.0);
                ui.label(RichText::new("→").size(32.0).color(Color32::from_rgb(142, 142, 147)));
                ui.add_space(20.0);
                
                self.render_stage_box(ui, "WAFER", vec![
                    "Not yet implemented".to_string(),
                    "Phase 3".to_string(),
                ], Color32::from_rgb(142, 142, 147));
            });
            
            ui.add_space(32.0);
            
            ui.label(
                RichText::new(format!("Simulation Time: {:.2} μs", self.simulation_state.simulation_time_us))
                    .size(14.0)
                    .color(Color32::from_rgb(142, 142, 147))
            );
        });
    }

    fn render_stage_box(&self, ui: &mut egui::Ui, title: &str, stats: Vec<String>, accent: Color32) {
        egui::Frame::none()
            .fill(Color32::WHITE)
            .stroke(Stroke::new(1.0, Color32::from_rgb(229, 229, 234)))
            .rounding(Rounding::same(8.0))
            .inner_margin(egui::Margin::same(16.0))
            .show(ui, |ui| {
                ui.set_min_width(160.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(title)
                            .size(12.0)
                            .color(accent)
                            .strong()
                    );
                    ui.add_space(8.0);
                    for stat in stats {
                        ui.label(
                            RichText::new(stat)
                                .size(13.0)
                                .color(Color32::from_rgb(60, 60, 67))
                        );
                    }
                });
            });
    }

    fn render_metric_card(&self, ui: &mut egui::Ui, title: &str, value: String, subtitle: String) {
        egui::Frame::none()
            .fill(Color32::WHITE)
            .stroke(Stroke::new(1.0, Color32::from_rgb(229, 229, 234)))
            .rounding(Rounding::same(8.0))
            .inner_margin(egui::Margin::same(16.0))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(title)
                            .size(11.0)
                            .color(Color32::from_rgb(142, 142, 147))
                    );
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(value)
                            .size(32.0)
                            .color(Color32::from_rgb(60, 60, 67))
                            .strong()
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(subtitle)
                            .size(12.0)
                            .color(Color32::from_rgb(142, 142, 147))
                    );
                });
            });
    }

    fn render_source_panel(&self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.label(RichText::new("Source").size(16.0).color(Color32::from_rgb(60, 60, 67)).strong());
            ui.add_space(12.0);
            
            self.render_metric_card(
                ui,
                "DROPLETS",
                format!("{}", self.simulation_state.droplet_count),
                "generated".to_string(),
            );
            
            ui.add_space(12.0);
            
            self.render_metric_card(
                ui,
                "PLASMA",
                format!("{}", self.simulation_state.plasma_count),
                "active events".to_string(),
            );
        });
    }

    fn render_optics_panel(&self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.label(RichText::new("Optics").size(16.0).color(Color32::from_rgb(60, 60, 67)).strong());
            ui.add_space(12.0);
            
            self.render_metric_card(
                ui,
                "PHOTONS",
                format!("{}", self.simulation_state.active_photons),
                format!("{:.1} avg bounces", self.simulation_state.average_bounces),
            );
            
            ui.add_space(12.0);
            
            let efficiency = if self.simulation_state.total_reflections + self.simulation_state.total_absorptions > 0 {
                (self.simulation_state.total_reflections as f32 / 
                (self.simulation_state.total_reflections + self.simulation_state.total_absorptions) as f32) * 100.0
            } else {
                0.0
            };
            
            self.render_metric_card(
                ui,
                "EFFICIENCY",
                format!("{:.1}%", efficiency),
                format!("{} reflections", self.simulation_state.total_reflections),
            );
        });
    }

    fn render_thermal_panel(&self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.label(RichText::new("Thermal").size(16.0).color(Color32::from_rgb(60, 60, 67)).strong());
            ui.add_space(12.0);
            
            self.render_metric_card(
                ui,
                "MAX TEMP",
                format!("{:.1}K", self.simulation_state.max_temperature),
                format!("avg {:.1}K", self.simulation_state.avg_temperature),
            );
            
            ui.add_space(12.0);
            
            self.render_metric_card(
                ui,
                "HEAT LOAD",
                format!("{:.2}kW", self.simulation_state.total_heat_energy / 1000.0),
                "total energy".to_string(),
            );
        });
    }

    fn render_charts(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.allocate_ui(egui::vec2(ui.available_width() / 2.0 - 8.0, 200.0), |ui| {
                self.render_photon_chart(ui);
            });
            
            ui.add_space(16.0);
            
            ui.allocate_ui(egui::vec2(ui.available_width(), 200.0), |ui| {
                self.render_temperature_chart(ui);
            });
        });
    }

    fn render_photon_chart(&self, ui: &mut egui::Ui) {
        egui::Frame::none()
            .fill(Color32::WHITE)
            .stroke(Stroke::new(1.0, Color32::from_rgb(229, 229, 234)))
            .rounding(Rounding::same(8.0))
            .inner_margin(egui::Margin::same(12.0))
            .show(ui, |ui| {
                ui.label(RichText::new("Active Photon Packets").size(13.0).color(Color32::from_rgb(60, 60, 67)));
                
                let points: PlotPoints = self.stats_history.time_points
                    .iter()
                    .zip(self.stats_history.photon_counts.iter())
                    .map(|(x, y)| [*x, *y])
                    .collect();
                
                let line = Line::new(points)
                    .color(Color32::from_rgb(0, 122, 255))
                    .width(2.0);
                
                Plot::new("photon_plot")
                    .height(150.0)
                    .show_axes([false, true])
                    .show_grid([false, true])
                    .show(ui, |plot_ui| plot_ui.line(line));
            });
    }

    fn render_temperature_chart(&self, ui: &mut egui::Ui) {
        egui::Frame::none()
            .fill(Color32::WHITE)
            .stroke(Stroke::new(1.0, Color32::from_rgb(229, 229, 234)))
            .rounding(Rounding::same(8.0))
            .inner_margin(egui::Margin::same(12.0))
            .show(ui, |ui| {
                ui.label(RichText::new("Mirror Temperature").size(13.0).color(Color32::from_rgb(60, 60, 67)));
                
                let max_points: PlotPoints = self.stats_history.time_points
                    .iter()
                    .zip(self.stats_history.max_temps.iter())
                    .map(|(x, y)| [*x, *y])
                    .collect();
                
                let avg_points: PlotPoints = self.stats_history.time_points
                    .iter()
                    .zip(self.stats_history.avg_temps.iter())
                    .map(|(x, y)| [*x, *y])
                    .collect();
                
                let max_line = Line::new(max_points)
                    .color(Color32::from_rgb(255, 59, 48))
                    .width(2.0);
                
                let avg_line = Line::new(avg_points)
                    .color(Color32::from_rgb(255, 149, 0))
                    .width(2.0);
                
                Plot::new("temp_plot")
                    .height(150.0)
                    .show_axes([false, true])
                    .show_grid([false, true])
                    .show(ui, |plot_ui| {
                        plot_ui.line(max_line);
                        plot_ui.line(avg_line);
                    });
            });
    }
}