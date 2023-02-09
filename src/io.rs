use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::vec::Vec;

pub fn read_words_from_file(filename: &PathBuf) -> Option<Vec<String>> {
    match File::open(filename) {
        Ok(file) => {
            let reader = BufReader::new(file);
            let mut lines = vec![];
            for line in reader.lines() {
                let line = line.unwrap();
                if line.contains("\r\n") {
                    for line in line.split("\r\n") {
                        lines.push(line.to_owned());
                    }
                } else if line.contains("\n") {
                    for line in line.split("\n") {
                        lines.push(line.to_owned());
                    }
                } else if line.contains("\r") {
                    for line in line.split("\r") {
                        lines.push(line.to_owned());
                    }
                } else {
                    lines.push(line);
                }
            }
            Some(lines)
        }
        Err(_) => { None }
    }
}