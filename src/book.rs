use fs4::fs_std::FileExt;
use regex::bytes::Regex;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs::create_dir_all;
use std::io;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use walkdir::WalkDir;

use crate::book_data::BookData;

#[derive(Clone)]
pub struct Book {
    pub id: u32,
    pub data: Option<BookData>,
    pub file: Arc<File>,
    pub description_hovered: bool,
}

pub fn load_data(data_path: &Path) -> Vec<Book> {
    if data_path.try_exists().is_err() {
        let result = create_dir_all(data_path);

        if result.is_err() {
            eprintln!(
                "Failed to create data directory:\n{}",
                result.unwrap_err().downcast::<io::Error>().unwrap_err()
            );
        }
    }

    let mut file_names: Vec<String> = vec![];
    let files_iterator = WalkDir::new(data_path)
        .max_depth(1)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|e| {
            e.metadata().unwrap().is_file()
                && e.path().extension().map_or(false, |ext| ext == "toml")
        });

    for entry in files_iterator {
        file_names.push(entry.file_name().to_str().unwrap().to_string())
    }

    let filename_regex = Regex::new(r"^\d{5}.toml$").unwrap();

    file_names.retain(|f| filename_regex.is_match(&f.as_bytes()));

    let mut file_list = vec![];
    for file_name in file_names {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(data_path.join(file_name.clone()))
            .unwrap();

        if file.try_lock_exclusive().unwrap() == false {
            eprintln!(
                "Failed to acquire lock for file: {:#?}",
                data_path.join(file_name)
            );
            continue;
        }

        file_list.push((file, file_name));
    }

    let mut books: Vec<Book> = Vec::new();
    for (mut file, file_name) in file_list {
        let read_result = read_book_from_file(&mut file);
        // handle errors
        if let Err((err, file_content)) = &read_result {
            if err.is::<std::io::Error>() {
                eprintln!(
                    "Failed to read file content: {:#?}",
                    data_path.join(&file_name)
                );
            }
            if err.is::<toml::de::Error>() {
                eprintln!(
                    "Could'nt parse toml file {:#?} with content:\n\n{} \nError: \n{}",
                    data_path.join(&file_name),
                    file_content,
                    err
                );
            }
            continue;
        }
        // end of handling errors
        let deserialized = read_result.unwrap();

        let id = file_name[..5].parse().unwrap();
        let book = Book {
            id,
            data: Option::Some(deserialized),
            file: Arc::new(file),
            description_hovered: false,
        };
        books.push(book);
    }
    books
}

pub fn read_book_from_file(
    file: &mut File,
) -> Result<BookData, (Box<dyn std::error::Error>, String)> {
    let mut file_content = String::new();
    let read_result = file.read_to_string(&mut file_content);

    if let Err(err) = read_result {
        return Err((Box::new(err), file_content));
    }

    let deserialized: Result<BookData, _> = toml::from_str(&file_content);

    if let Err(err) = deserialized {
        return Err((Box::new(err), file_content));
    }

    Ok(deserialized.unwrap())
}
