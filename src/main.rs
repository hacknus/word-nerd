mod gui;
mod io;

use crate::gui::{MyApp, SettingsContainer, StepDir};
use eframe::egui;
use eframe::epaint::text::{FontInsert, FontPriority, InsertFontFamily};
use io::read_words_from_file;
use preferences::{AppInfo, Preferences};
use rand::Rng;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, RwLock};
use std::thread;
use std::time::Duration;

const APP_INFO: AppInfo = AppInfo {
    name: "Word Nerd",
    author: "Linus Leo Stöckli, Lea Höfliger",
};
const HISTORY_SIZE: usize = 512;

fn add_font(ctx: &egui::Context) {
    ctx.add_font(FontInsert::new(
        "my_font",
        egui::FontData::from_static(include_bytes!("../fonts/DCH-Basisschrift.ttf")),
        vec![InsertFontFamily {
            family: egui::FontFamily::Name("my_font".into()),
            priority: FontPriority::Highest,
        }],
    ));
}

fn main_thread(
    rate_lock: Arc<RwLock<f32>>,
    random_lock: Arc<RwLock<bool>>,
    running_lock: Arc<RwLock<bool>>,
    mode_lock: Arc<RwLock<bool>>,
    word_lock: Arc<RwLock<String>>,
    step_rx: Receiver<StepDir>,
    load_rx: Receiver<PathBuf>,
) {
    // reads data from mutex, samples and saves if needed
    let mut rate = 120.0;
    let mut running = false;
    let mut randomizer = false;
    let mut idx = 0;
    let mut i = 0;
    let mut word;
    let file_path = PathBuf::from("abc.txt");
    let mut words = vec!["keine gültige Datei gefunden".to_string()];
    match read_words_from_file(&file_path) {
        None => {}
        Some((w, mode)) => {
            words = w;
            if let Ok(mut write_guard) = mode_lock.write() {
                *write_guard = mode;
            }
        }
    }
    let mut history = vec![idx];
    loop {
        if let Ok(read_guard) = running_lock.read() {
            running = *read_guard;
        }

        if let Ok(read_guard) = rate_lock.read() {
            rate = read_guard.clone();
        }

        if let Ok(read_guard) = random_lock.read() {
            randomizer = *read_guard;
        }

        match load_rx.recv_timeout(Duration::from_millis(1)) {
            Ok(fp) => {
                // load file
                if let Some((w, mode)) = read_words_from_file(&fp) {
                    words = w;
                    if let Ok(mut write_guard) = mode_lock.write() {
                        *write_guard = mode;
                    }
                }
            }
            Err(..) => (),
        }

        if running {
            if randomizer {
                // get random word out of words
                idx = rand::rng().random_range(0..words.len());
                if *history.last().unwrap() == idx {
                    idx = (idx + 1) % words.len();
                }
            } else {
                idx = (idx + 1) % words.len();
            }

            word = words[idx].clone();
            history.push(idx);
            i = history.len() - 1;

            if let Ok(mut write_guard) = word_lock.write() {
                *write_guard = word.clone();
            }
            std::thread::sleep(Duration::from_millis((60.0 / rate * 1000.0) as u64));
        } else {
            match step_rx.recv_timeout(Duration::from_millis(1)) {
                Ok(step) => {
                    match step {
                        StepDir::FORWARD => {
                            if i < history.len() - 1 {
                                i += 1;
                                idx = history[i];
                            } else {
                                if randomizer {
                                    // get random word out of words
                                    idx = rand::rng().random_range(0..words.len());
                                    if *history.last().unwrap() == idx {
                                        idx = (idx + 1) % words.len();
                                    }
                                } else {
                                    idx = (idx + 1) % words.len();
                                }
                                history.push(idx);
                                i = history.len() - 1;
                            }
                        }
                        StepDir::BACKWARD => {
                            if i > 0 {
                                i -= 1;
                                idx = history[i];
                            } else {
                                if idx > 0 {
                                    idx -= 1;
                                } else {
                                    idx = words.len() - 1;
                                }
                                history.push(idx);
                                i = history.len() - 1;
                            }
                        }
                    }
                    word = words[idx].clone();
                    if let Ok(mut write_guard) = word_lock.write() {
                        *write_guard = word.clone();
                    }
                }
                Err(..) => (),
            }
        }
        if history.len() > HISTORY_SIZE {
            let len = history.len();
            history = history[len - HISTORY_SIZE..len].to_vec();
        }
    }
}

fn main() {
    let mut gui_settings = SettingsContainer::default();
    let prefs_key = "config/gui";
    if let Ok(loaded) = SettingsContainer::load(&APP_INFO, prefs_key) {
        gui_settings = loaded;
    } else {
        let _ = gui_settings.save(&APP_INFO, prefs_key);
    }

    // shared state
    let running_lock = Arc::new(RwLock::new(false));
    let random_lock = Arc::new(RwLock::new(gui_settings.random));
    let rate_lock = Arc::new(RwLock::new(gui_settings.rate));
    let word_lock = Arc::new(RwLock::new("Hallo!".to_string()));
    let mode_lock = Arc::new(RwLock::new(false)); // false = word-per-line by default

    // channels
    let (load_tx, load_rx): (Sender<PathBuf>, Receiver<PathBuf>) = mpsc::channel();
    let (step_tx, step_rx): (Sender<StepDir>, Receiver<StepDir>) = mpsc::channel();

    let main_rate_lock = rate_lock.clone();
    let main_random_lock = random_lock.clone();
    let main_word_lock = word_lock.clone();
    let main_running_lock = running_lock.clone();
    let main_mode_lock = mode_lock.clone();

    println!("starting main thread..");
    thread::spawn(move || {
        main_thread(
            main_rate_lock,
            main_random_lock,
            main_running_lock,
            main_mode_lock,
            main_word_lock,
            step_rx,
            load_rx,
        );
    });

    // prepare UI
    let options = eframe::NativeOptions {
        ..Default::default()
    };

    let gui_rate_lock = rate_lock.clone();
    let gui_random_lock = random_lock.clone();
    let gui_word_lock = word_lock.clone();
    let gui_running_lock = running_lock.clone();
    let gui_mode_lock = mode_lock.clone();

    // send initial file path to background thread
    let _ = load_tx.send(gui_settings.file_path.clone());

    eframe::run_native(
        "Word Nerd",
        options,
        Box::new(move |cc| {
            add_font(&cc.egui_ctx);
            Ok(Box::new(MyApp::new(
                gui_random_lock,
                gui_rate_lock,
                gui_running_lock,
                gui_word_lock,
                gui_mode_lock,
                gui_settings,
                step_tx,
                load_tx,
            )))
        }),
    )
    .expect("GUI did not start");
}
