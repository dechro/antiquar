use fs4::fs_std::FileExt;
use gpui::http_client::anyhow;
use regex::bytes::Regex;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::env;
use std::fs::create_dir_all;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use toml::de::Error;
use walkdir::WalkDir;

use rust_embed::RustEmbed;

use gpui::*;
use gpui_component::*;

use crate::ui::main_window::MainWindow;

pub mod book_data;
pub mod ui;

use crate::book_data::BookData;

#[derive(RustEmbed)]
#[folder = "./assets"]
#[include = "icons/**/*.svg"]
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }

        Self::get(path)
            .map(|f| Some(f.data))
            .ok_or_else(|| anyhow!("could not find asset at path \"{path}\""))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(Self::iter().filter_map(|p| p.starts_with(path).then(|| p.into())).collect())
    }
}

fn main() {
    let config = load_config();
    let books = load_data(Path::new(&config.datapath));

    let app = Application::new().with_assets(Assets);
    app.run(move |app| {
        let books = books.clone().iter().map(|element| app.new(|cx| element.clone())).collect();
        let books = app.new(|cx| books);
        gpui_component::init(app);
        app.spawn(async move |cx| {
            cx.open_window(WindowOptions::default(), |window, cx| {
                let view = cx.new(|cx| MainWindow::new(window, cx, books));
                cx.new(|cx| Root::new(view.into(), window, cx))
            })
        })
        .detach();
    })
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    datapath: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            datapath: "/home/robertd/test/Antiquar".to_string(),
        }
    }
}

fn load_config() -> Config {
    let args = env::args().collect::<Vec<String>>(); // Get command line arguments

    // Try to load config from file

    if !args.get(1).is_some_and(|f| Path::new(f).exists()) {
        println!("No config file provided, using default values");
        let config: Config = Default::default();
        return config;
    }
    // If no argument or an invalid path is passed, use default
    let mut file_content = String::new(); // Buffer for file content

    let open_options = OpenOptions::new().open(Path::new(args.get(1).unwrap()));
    if let v @ Err(_) = open_options {
        eprintln!("Couldn't open config file: {} \n{}", args.get(1).unwrap(), v.unwrap_err());
        let config: Config = Default::default();
        return config;
    }

    let mut open_options = open_options.unwrap();

    let read_result = open_options.read_to_string(&mut file_content);
    if let v @ Err(_) = read_result {
        eprintln!("Couldn't read contents of file: {} \n{}", args.get(1).unwrap(), v.unwrap_err());
        let config: Config = Default::default();
        return config;
    };

    let config = toml::from_str(&file_content);
    if let v @ Err(_) = config {
        eprintln!("Couldn't parse contents of file: {} \n{}", args.get(1).unwrap(), v.unwrap_err());
        let config: Config = Default::default();
        return config;
    };

    config.unwrap()
}

fn load_data(data_path: &Path) -> Vec<(u32, Option<BookData>, Arc<File>)> {
    if data_path.try_exists().is_err() {
        create_dir_all(data_path).unwrap_or_else(|e| {
            eprintln!(
                "Failed to create data directory:\n{}",
                e.downcast::<io::Error>().unwrap_err()
            );
        });
    }
    let mut filenames: Vec<String> = vec![];
    for entry in WalkDir::new(data_path)
        .max_depth(1)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|e| {
            e.metadata().unwrap().is_file()
                && e.path().extension().map_or(false, |ext| ext == "toml")
        })
    {
        filenames.push(entry.file_name().to_str().unwrap().to_string())
    }
    let filename_regex = Regex::new(r"^\d{5}.toml$").unwrap();
    filenames.retain(|f| filename_regex.is_match(&f.as_bytes()));
    let mut files = vec![];
    for f in filenames {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(data_path.join(f.clone()))
            .unwrap();
        if file.try_lock_exclusive().unwrap() == false {
            eprintln!("Failed to acquire lock for file: {:#?}", data_path.join(f));
            panic!()
        }

        files.push((file, f));
    }
    let mut books: Vec<(u32, Option<BookData>, Arc<File>)> = Vec::new();
    for mut file in files {
        let mut content = String::new();
        file.0.read_to_string(&mut content).expect("");
        let deserialized: Result<BookData, Error> = toml::from_str(&content);
        match deserialized {
            Ok(_) => {
                books.push((
                    file.1[..5].parse().unwrap(),
                    Option::Some(deserialized.unwrap()),
                    Arc::new(file.0),
                ));
            }
            Err(_) => {
                eprintln!(
                    "Could'nt parse toml file {:#?} with content:\n\n{} \nError: \n{}",
                    data_path.join(&file.1),
                    content,
                    deserialized.unwrap_err()
                );
                books.push((file.1[..5].parse().unwrap(), Option::None, Arc::new(file.0)));
            }
        };
    }
    books
}
