use serde_json::Value;
use std::fs;
use std::path::PathBuf;
pub struct ConfFile {
    contents: String,
}

impl ConfFile {
    pub fn new(path: PathBuf) -> std::io::Result<Self> {
        let contents = fs::read_to_string(&path)?;
        Ok(Self { contents })
    }

    pub fn read(&self) -> String {
        self.contents.to_string()
    }
}

pub fn get_conf_data(conf: String, which: &str) -> String {
    let mut out = String::new();
    let data: Value = serde_json::from_str(&conf).expect("Failed to get data from json");

    if let Some(data_array) = data.as_array() {
        for entry in data_array {
            if let Some(target) = entry.get(which).and_then(|s| s.as_str()) {
                out = target.to_string();
            } else {
                println!("Couldn't find \"{}\" in your config file!", which);
            }
        }
    }

    out
}

pub fn get_border_color(conf: String) -> Vec<String> {
    let mut color = Vec::new();
    let data: Value = serde_json::from_str(&conf).expect("Failed to get data from json");

    if let Some(data_array) = data.as_array() {
        for entry in data_array {
            if let Some(targets) = entry.get("border_color").and_then(|s| s.as_array()) {
                for target in targets {
                    if let Some(color_cute) = target.as_str() {
                        color.push(color_cute.to_string());
                    }
                }
            }
        }
    }

    color
}

pub fn string_to_i32(input: String, which: &str) -> i32 {
    let out;
    match input.parse::<i32>() {
        Ok(number) => {
            out = number;
        }
        Err(_) => {
            if which == "height" {
                println!("Couldn't parse a string to a number");
                println!("Going with the default height: 800");
                out = 800;
            } else if which == "width" {
                println!("Couldn't parse a string to a number");
                println!("Going with the default width: 600");
                out = 600;
            } else {
                out = 0;
            }
        }
    }
    out
}

pub fn string_to_u32(input: String) -> u32 {
    let out;
    match input.parse::<u32>() {
        Ok(number) => {
            out = number;
        }
        Err(_) => {
            out = 0;
        }
    }
    out
}
