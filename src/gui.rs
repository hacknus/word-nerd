use crate::APP_INFO;
use eframe::egui::{Align2, DragValue, FontFamily, FontId, Pos2, RichText, Sense, Vec2};
use eframe::{egui, Storage};
use egui_theme_switch::global_theme_switch;
use preferences::Preferences;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};
use std::time::Instant;

pub enum StepDir {
    FORWARD,
    BACKWARD,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct SettingsContainer {
    pub rate: f32,
    pub random: bool,
    pub idx: isize,
    pub file_path: PathBuf,
    pub font_size: f32,
    pub x: f32,
    pub y: f32,
}

impl SettingsContainer {
    pub fn default() -> SettingsContainer {
        return SettingsContainer {
            rate: 120.0,
            idx: 0,
            random: false,
            file_path: PathBuf::from("abc.txt"),
            font_size: 50.0,
            x: 450.0,
            y: 900.0,
        };
    }
}

pub struct MyApp {
    running: bool,
    word: String,
    conf: SettingsContainer,
    random_lock: Arc<RwLock<bool>>,
    rate_lock: Arc<RwLock<f32>>,
    running_lock: Arc<RwLock<bool>>,
    word_lock: Arc<RwLock<String>>,
    mode_lock: Arc<RwLock<bool>>,
    step_tx: Sender<StepDir>,
    load_tx: Sender<PathBuf>,

    // scrolling animation state (for poem/paragraph mode)
    scroll_offset: f32,
    last_instant: Instant,
}

impl MyApp {
    pub fn new(
        random_lock: Arc<RwLock<bool>>,
        rate_lock: Arc<RwLock<f32>>,
        running_lock: Arc<RwLock<bool>>,
        word_lock: Arc<RwLock<String>>,
        mode_lock: Arc<RwLock<bool>>,
        conf: SettingsContainer,
        step_tx: Sender<StepDir>,
        load_tx: Sender<PathBuf>,
    ) -> Self {
        Self {
            running: false,
            word: "Hallo".to_string(),
            conf,
            random_lock,
            rate_lock,
            running_lock,
            word_lock,
            mode_lock,
            step_tx,
            load_tx,
            scroll_offset: 0.0,
            last_instant: Instant::now(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(ui.available_size().y * 0.3);
            if let Ok(read_guard) = self.word_lock.read() {
                self.word = read_guard.clone()
            }

            let poem_mode = if let Ok(read_guard) = self.mode_lock.read() {
                *read_guard
            } else {
                false
            };

            if poem_mode && self.running {
                // Scrolling animation: move the single-line text from left to right
                let avail = ui.available_size();
                let height = (self.conf.font_size * 1.4).max(30.0);
                let area_size = Vec2::new(avail.x, height);
                let (rect, _response) = ui.allocate_exact_size(area_size, Sense::hover());

                // Timing
                let now = Instant::now();
                let dt = now.duration_since(self.last_instant).as_secs_f32();
                self.last_instant = now;

                // Speed: use conf.rate as pixels/sec
                let speed_px_per_sec = self.conf.rate;
                self.scroll_offset -= speed_px_per_sec * dt;

                // Approximate text pixel width
                let char_width_est = self.conf.font_size * 0.6; // heuristic
                let text_px_width = (self.word.chars().count() as f32) * char_width_est;
                let gap = rect.width().max(40.0); // gap between repetitions
                let total_cycle = text_px_width + gap;

                if total_cycle > 0.0 {
                    // Wrap offset so it loops
                    self.scroll_offset = self.scroll_offset % total_cycle;
                } else {
                    self.scroll_offset = 0.0;
                }

                // Compute base x so text moves left-to-right so text starts centered when scroll_offset = 0.0
                let base_x = rect.center().x + self.scroll_offset;

                // Use `with_clip_rect` to temporarily set the clipping rectangle
                // Create a new Painter with the specified clipping rectangle
                let clipped_painter = ui.painter().with_clip_rect(rect);

                // Primary copy
                let y = rect.center().y - (self.conf.font_size / 2.0);
                clipped_painter.text(
                    Pos2::new(base_x, y),
                    Align2::LEFT_TOP,
                    &self.word,
                    FontId::new(self.conf.font_size, FontFamily::Name("my_font".into())),
                    ui.style().visuals.strong_text_color(),
                );

                // Second copy (shifted by +total_cycle) so when primary leaves we still have text
                clipped_painter.text(
                    Pos2::new(base_x + total_cycle, y),
                    Align2::LEFT_TOP,
                    &self.word,
                    FontId::new(self.conf.font_size, FontFamily::Name("my_font".into())),
                    ui.style().visuals.strong_text_color(),
                );

                ctx.request_repaint();

            } else {
                self.last_instant = Instant::now();

                if self.word.chars().count() >= 10 {
                    ui.vertical_centered(|ui| {
                        let font_id =
                            FontId::new(self.conf.font_size, FontFamily::Name("my_font".into()));
                        ui.label(RichText::new("Drücke auf Start...").font(font_id).strong());
                    });
                } else {
                    ui.vertical_centered(|ui| {
                        let font_id =
                            FontId::new(self.conf.font_size, FontFamily::Name("my_font".into()));
                        ui.label(RichText::new(&self.word).font(font_id).strong());
                    });
                }
            }
            ui.vertical_centered(|ui| {
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(50.0);

                    if ui.button("Datei öffnen").clicked() {
                        match rfd::FileDialog::new().pick_file() {
                            Some(path) => {
                                self.conf.file_path = path;
                            }
                            None => self.conf.file_path = PathBuf::new(),
                        }

                        self.running = false;
                        self.word = "Drücke auf Start...".to_string();
                        if let Ok(mut guard) = self.word_lock.write() {
                            *guard = self.word.clone();
                        }

                        println!("opening a new file");
                        let _ = self.load_tx.send(self.conf.file_path.clone());
                    }

                    ui.add_space(10.0);

                    let suffix = if poem_mode {
                        " cpm"
                    } else {
                        " wpm"
                    };

                    ui.add(
                        DragValue::new(&mut self.conf.rate)
                            .fixed_decimals(0)
                            .range(10.0..=800.0)
                            .suffix(suffix),
                    );

                    ui.add_space(5.0);

                    ui.label(RichText::new("Frequenz:").size(20.0).strong());

                    if poem_mode {
                        ui.add_space(5.0);
                        if ui.button(RichText::new("Reset").size(20.0)).clicked() {
                            self.scroll_offset = 0.0;
                        }
                        ui.add_space(5.0);
                    } else {
                        ui.add_space(10.0);
                    }


                    let b_text = if self.running {
                        RichText::new("Stopp").size(20.0).strong()
                    } else {
                        RichText::new("Start").size(20.0).strong()
                    };

                    let mut space_pressed = false;
                    if ui.input(|i| i.key_released(egui::Key::Space)) {
                        space_pressed = true;
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::ArrowUp) || i.key_pressed(egui::Key::ArrowDown)) {
                        let shift = ui.input(|i| i.modifiers.shift);
                        let step = if shift { 10.0 } else { 1.0 };

                        if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) && self.conf.rate <= 800.0 - step {
                            self.conf.rate += step;
                        }
                        if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) && self.conf.rate >= 10.0 + step {
                            self.conf.rate -= step;
                        }
                    }

                    if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                        let _ = self.step_tx.send(StepDir::BACKWARD);
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                        let _ = self.step_tx.send(StepDir::FORWARD);
                    }

                    if ui.button(b_text).clicked() || space_pressed {
                        self.running = !self.running;
                    }
                });
            });
            ui.add_space(ui.available_size().y - 25.0);
            ui.horizontal(|ui| {
                global_theme_switch(ui);
                ui.add_space(10.0);
                ui.label("  Schriftgrösse: ");
                ui.add(egui::Slider::new(&mut self.conf.font_size, 40.0..=200.0));
                ui.add_space(10.0);
                ui.add_enabled_ui(!poem_mode, |ui| {
                    ui.checkbox(&mut self.conf.random, "Random").on_hover_text("Zufällige Wörter im Wort/Buchstaben-Modus. Deaktiviert im Gedicht/Paragraph-Modus.");
                })
            });
        });

        if let Ok(mut write_guard) = self.rate_lock.write() {
            *write_guard = self.conf.rate;
        }
        if let Ok(mut write_guard) = self.running_lock.write() {
            *write_guard = self.running;
        }
        if let Ok(mut write_guard) = self.random_lock.write() {
            *write_guard = self.conf.random;
        }
        self.conf.x = ctx.used_size().x;
        self.conf.y = ctx.used_size().y;
    }

    fn save(&mut self, _storage: &mut dyn Storage) {
        let prefs_key = "config/gui";
        match self.conf.save(&APP_INFO, prefs_key) {
            Ok(_) => {}
            Err(err) => {
                println!("gui settings save failed: {:?}", err);
            }
        }
    }
}
