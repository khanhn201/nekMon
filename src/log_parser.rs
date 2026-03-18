use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct Config {
    block: Vec<Block>,
}

#[derive(Debug, Deserialize)]
struct Block {
    contains: Option<String>,
    #[serde(default)]
    start_new_record: bool,
    columns: Vec<Column>,
}

#[derive(Debug, Deserialize)]
struct Column {
    field: String,
    index: usize,
    #[serde(default)]
    default: Option<f64>,
}

pub type Record = HashMap<String, f64>;

#[cfg(feature = "ssr")]
pub fn parse(log_path: &str, config_path: &str) -> Vec<Record> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use toml;

    let float_regex = regex::Regex::new(r"[+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?").unwrap();
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

            if !matches {
                continue;
            }

            if block.start_new_record {
                if let Some(r) = current.take() {
                    results.push(r);
                }

                let mut record = HashMap::new();
                for block in &config.block {
                    for col in &block.columns {
                        record.insert(col.field.clone(), col.default.unwrap_or(0.0));
                    }
                }
                current = Some(record);
            }

            if let Some(ref mut record) = current {
                let floats: Vec<f64> = float_regex
                    .find_iter(&line)
                    .filter_map(|m| m.as_str().parse().ok())
                    .collect();

                for col in &block.columns {
                    match floats.get(col.index - 1) {
                        Some(&val) => {
                            record.insert(col.field.clone(), val);
                        }
                        None => {}
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
