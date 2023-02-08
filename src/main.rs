#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// hide console window on Windows in release
extern crate serde;
extern crate preferences;
extern crate core;

mod gui;
mod toggle;
mod io;

use std::cmp::max;
use std::path::PathBuf;
use std::thread;
use eframe::egui::{vec2, Visuals};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, mpsc, RwLock};
use std::time::{Duration};
use preferences::{AppInfo, Preferences};
use rand::random;
use io::read_words_from_file;

use crate::gui::{GuiSettingsContainer, MyApp};

const APP_INFO: AppInfo = AppInfo { name: "Word Nerd", author: "Linus Leo Stöckli" };


fn main_thread(rate_lock: Arc<RwLock<f32>>,
               running_lock: Arc<RwLock<bool>>,
               word_lock: Arc<RwLock<String>>,
               load_rx: Receiver<PathBuf>,
) {
    // reads data from mutex, samples and saves if needed
    let mut file_path = PathBuf::from("abc.txt");
    let mut rate = 120.0;
    let mut running = false;
    let mut word = "A".to_string();
    let mut words = read_words_from_file(&file_path);
    loop {
        if let Ok(read_guard) = running_lock.read() {
            running = read_guard.clone();
        }

        if let Ok(read_guard) = rate_lock.read() {
            rate = read_guard.clone();
        }

        match load_rx.recv_timeout(Duration::from_millis(10)) {
            Ok(fp) => {
                // load file
                words.push(word.clone());
            }
            Err(..) => ()
        }

        if running {
            // get random word out of words

            let idx = random::<usize>() % words.len();
            word = words[idx].clone();

            if let Ok(mut write_guard) = word_lock.write() {
                *write_guard = word.clone();
            }
        }


        std::thread::sleep(Duration::from_millis((60.0 / rate * 1000.0) as u64));
    }
}

fn main() {
    let mut gui_settings = GuiSettingsContainer::default();
    let prefs_key = "config/gui";
    let load_result = GuiSettingsContainer::load(&APP_INFO, prefs_key);
    if load_result.is_ok() {
        gui_settings = load_result.unwrap();
    } else {
        // save default settings
        match gui_settings.save(&APP_INFO, prefs_key) {
            Ok(_) => {}
            Err(_) => {
                println!("failed to save gui_settings");
            }
        }
    }

    let running_lock = Arc::new(RwLock::new(false));
    let rate_lock = Arc::new(RwLock::new(gui_settings.rate));
    let word_lock = Arc::new(RwLock::new("HELLO".to_string()));

    let (load_tx, load_rx): (Sender<PathBuf>, Receiver<PathBuf>) = mpsc::channel();

    let main_rate_lock = rate_lock.clone();
    let main_word_lock = word_lock.clone();
    let main_running_lock = running_lock.clone();

    println!("starting main thread..");
    let main_thread_handler = thread::spawn(|| {
        main_thread(main_rate_lock,
                    main_running_lock,
                    main_word_lock,
                    load_rx);
    });


    let options = eframe::NativeOptions {
        drag_and_drop_support: true,
        initial_window_size: Option::from(vec2(gui_settings.x, gui_settings.y)),
        ..Default::default()
    };

    let gui_rate_lock = rate_lock.clone();
    let gui_word_lock = word_lock.clone();
    let gui_running_lock = running_lock.clone();

    let visuals;
    if gui_settings.dark_mode{
        visuals = Visuals::dark();
    } else {
        visuals = Visuals::light();
    }

    eframe::run_native(
        "Word Nerd",
        options,
        Box::new(|_cc| {
            _cc.egui_ctx.set_visuals(visuals);
            Box::new(MyApp::new(
                gui_rate_lock,
                gui_running_lock,
                gui_word_lock,
                gui_settings,
                load_tx
            ))
        }),
    );
}
