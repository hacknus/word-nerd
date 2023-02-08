use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::vec::Vec;

pub fn read_words_from_file(filename: &PathBuf) -> Vec<String> {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    reader
        .lines()
        .map(|l| l.unwrap())
        .collect::<Vec<String>>()
}