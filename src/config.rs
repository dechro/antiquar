// SPDX-License-Identifier: MIT

use std::env;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::Path;

use crate::book_data::BookData;

use cosmic::cosmic_config::{self, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, CosmicConfigEntry, Eq, PartialEq, Serialize, Deserialize)]
#[version = 1]
pub struct Config {
    pub data_path: String,
}

impl Default for Config {
    fn default() -> Self {
        let mut datapath = env::var("XDG_DATA_HOME");
        if datapath.is_err() {
            let home_dir = env::var("HOME").unwrap();
            datapath = Ok(format!("{}/.local/share", home_dir));
        }

        let datapath = format!("{}/antiquar", datapath.unwrap());
        Config {
            data_path: datapath,
        }
    }
}

//pub fn load_config() -> Config {
//    let args = env::args().collect::<Vec<String>>(); // Get command line arguments
//
//    // Try to load config from file
//
//    if !args.get(1).is_some_and(|f| Path::new(f).exists()) {
//        println!("No config file provided, using default values");
//        let config: Config = Default::default();
//        return config;
//    }
//    // If no argument or an invalid path is passed, use default
//    let mut file_content = String::new(); // Buffer for file content
//
//    let open_options = OpenOptions::new().open(Path::new(args.get(1).unwrap()));
//    if let v @ Err(_) = open_options {
//        eprintln!(
//            "Couldn't open config file: {} \n{}",
//            args.get(1).unwrap(),
//            v.unwrap_err()
//        );
//        let config: Config = Default::default();
//        return config;
//    }
//
//    let mut open_options = open_options.unwrap();
//
//    let read_result = open_options.read_to_string(&mut file_content);
//    if let v @ Err(_) = read_result {
//        eprintln!(
//            "Couldn't read contents of file: {} \n{}",
//            args.get(1).unwrap(),
//            v.unwrap_err()
//        );
//        let config: Config = Default::default();
//        return config;
//    };
//
//    let config = toml::from_str(&file_content);
//    if let v @ Err(_) = config {
//        eprintln!(
//            "Couldn't parse contents of file: {} \n{}",
//            args.get(1).unwrap(),
//            v.unwrap_err()
//        );
//        let config: Config = Default::default();
//        return config;
//    };
//
//    config.unwrap()
//}
//
