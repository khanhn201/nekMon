use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use toml;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Config {
    block: Vec<Block>,
}

#[derive(Debug, Deserialize)]
struct Block {
    name: String,
    contains: Option<String>,
    start_new_record: bool,
    columns: Vec<Column>,
}

#[derive(Debug, Deserialize)]
struct Column {
    field: String,
    index: usize,
}

type Record = HashMap<String, f64>;

fn parse(log_path: &str, config_path: &str) -> Vec<Record> {
    let config_str = std::fs::read_to_string(config_path).unwrap();
    let config: Config = toml::from_str(&config_str).unwrap();

    let file = File::open(log_path).unwrap();
    let reader = BufReader::new(file);

    let mut results = Vec::new();
    let mut current: Option<Record> = None;

    for line in reader.lines() {
        let line = line.unwrap();

        for block in &config.block {
            let matches = block
                .contains
                .as_ref()
                .map(|s| line.contains(s))
                .unwrap_or(true);

            if matches {
                if block.start_new_record {
                    if let Some(r) = current.take() {
                        results.push(r);
                    }
                    current = Some(HashMap::new());
                }

                if let Some(ref mut record) = current {
                    let parts: Vec<&str> = line.split_whitespace().collect();

                    for col in &block.columns {
                        if let Some(val) = parts.get(col.index) {
                            record.insert(
                                col.field.clone(),
                                val.parse::<f64>().expect("could not be parse to f64"),
                            );
                        }
                    }
                }
            }
        }
    }

    if let Some(r) = current {
        results.push(r);
    }

    results
}
