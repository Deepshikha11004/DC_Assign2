use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct Params {
    pub n: u32,
    pub k: u32,
    pub alpha: u32,
    pub beta: u32,
}

impl Params {
    pub fn new() -> Self {
        // todo!()

        let file = File::open("inp-params.txt").expect("Failed to open input file");

        // Create a buffered reader to read the file line by line
        let reader = BufReader::new(file);

        // Read the first line and split it into individual parameters
        let params: Vec<u32> = reader
            .lines()
            .next()
            .expect("Input file is empty")
            .expect("Failed to read line")
            .split_whitespace()
            .map(|param| param.parse().expect("Failed to parse parameter"))
            .collect();

        // Assign the parameters to variables
        Self {
            n: params[0],
            k: params[1],
            alpha: params[2],
            beta: params[3],
        }
    }
}

pub mod a;
pub mod b;

