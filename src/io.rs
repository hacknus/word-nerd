use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::vec::Vec;

pub fn read_words_from_file(filename: &PathBuf) -> Option<Vec<String>> {
    if let Ok(file) = File::open(filename) {
        let reader = BufReader::new(file);
        Some(reader
            .lines()
            .map(|l| match l {
                Ok(s) => {
                    s
                }
                Err(_) => {
                    "Fehler in Datei".to_string()
                }
            })
            .collect::<Vec<String>>())
    } else {
       None
    }
}