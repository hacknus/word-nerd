use core::f32;
use std::ops::RangeInclusive;
use std::path::PathBuf;
use std::sync::mpsc::{Sender};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use eframe::{egui, Storage};
use eframe::egui::panel::{Side};
use eframe::egui::plot::{Legend, Line, Plot, PlotPoints};
use eframe::egui::{FontId, FontFamily, RichText, global_dark_light_mode_buttons, Visuals, Direction, DragValue};
use eframe::egui::Key::Space;
use crate::toggle::toggle;
use preferences::{Preferences};
use crate::{APP_INFO, vec2};
use serde::{Deserialize, Serialize};


const MAX_FPS: f64 = 24.0;


#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct GuiSettingsContainer {
    pub rate: f32,
    pub x: f32,
    pub y: f32,
}

impl GuiSettingsContainer {
    pub fn default() -> GuiSettingsContainer {
        return GuiSettingsContainer {
            rate: 120.0,
            x: 450.0,
            y: 900.0,
        };
    }
}

pub struct MyApp {
    dark_mode: bool,
    running: bool,
    word: String,
    rate: f32,
    picked_path: PathBuf,
    gui_conf: GuiSettingsContainer,
    rate_lock: Arc<RwLock<f32>>,
    running_lock: Arc<RwLock<bool>>,
    word_lock: Arc<RwLock<String>>,
    load_tx: Sender<PathBuf>,
}

impl MyApp {
    pub fn new(rate_lock: Arc<RwLock<f32>>,
               running_lock: Arc<RwLock<bool>>,
               word_lock: Arc<RwLock<String>>,
               gui_conf: GuiSettingsContainer,
               load_tx: Sender<PathBuf>,
    ) -> Self {
        Self {
            dark_mode: true,
            running: false,
            word: "A".to_string(),
            rate: 120.0,
            picked_path: PathBuf::new(),
            gui_conf,
            rate_lock,
            running_lock,
            word_lock,
            load_tx,
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

            ui.vertical_centered(|ui| {
                ui.label(RichText::new(&self.word).size(50.0).strong());
            });
            ui.add_space(ui.available_size().y * 0.3);

            ui.vertical_centered(|ui| {
                let b_text;
                if self.running {
                    b_text = "Stopp";
                } else {
                    b_text = "Start";
                }
                let events = ui.input().events.clone();
                let mut space_pressed = false;
                for event in &events {
                    match event {
                        egui::Event::Key{key, pressed, modifiers} => {
                            if *key == Space && *pressed == false {
                                space_pressed = true;
                            }
                        },
                        _ => {}
                    }
                }

                if ui.button(b_text).clicked() || space_pressed {
                    self.running = !self.running;
                }
                ui.add_space(10.0);
                ui.label("Frequenz");
                ui.add(DragValue::new(&mut self.rate).clamp_range(10.0..=800.0).suffix(" wpm"));
            });
        });

        if let Ok(mut write_guard) = self.rate_lock.write() {
            *write_guard = self.rate.clone();
        }
        if let Ok(mut write_guard) = self.running_lock.write() {
            *write_guard = self.running.clone();
        }

        self.gui_conf.x = ctx.used_size().x;
        self.gui_conf.y = ctx.used_size().y;

        ctx.request_repaint();

        std::thread::sleep(Duration::from_millis((1000.0 / MAX_FPS) as u64));
    }

    fn save(&mut self, _storage: &mut dyn Storage) {
        let prefs_key = "config/gui";
        match self.gui_conf.save(&APP_INFO, prefs_key) {
            Ok(_) => {}
            Err(err) => {
                println!("gui settings save failed: {:?}", err);
            }
        }
    }
}