use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::vec::Vec;

fn is_word_per_line(lines: &[String]) -> bool {
    // true when every non-empty line contains at most one whitespace-separated token
    lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .all(|l| l.trim().split_whitespace().count() <= 1)
}

pub fn read_words_from_file(filename: &PathBuf) -> Option<(Vec<String>, bool)> {
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
            if !is_word_per_line(&lines) {
                // concatenate all lines with "   " as separator
                lines = vec![lines
                    .iter()
                    .filter(|l| !l.trim().is_empty())
                    .cloned()
                    .collect::<Vec<String>>()
                    .join("   ")];
                Some((lines, true))
            } else {
                Some((lines, false))
            }
        }
        Err(_) => None,
    }
}
