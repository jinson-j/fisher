use std::path::PathBuf;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::io::BufRead;
use std::io::BufReader;

use faiss::vector_transform::VectorTransform;
use faiss::*;


pub fn setup_vector_store(directory: PathBuf) {
    let vs_dir = directory.join(".vs");

    if !vs_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&vs_dir) {
            eprintln!("Failed to create .vs directory: {}", e);
        }
    } 

    let files = get_files(directory.clone());
    let files_txt = get_files_txt(directory.clone());

    for file in files {
        if !files_txt.contains(&file.to_str().unwrap().to_string()) {
            add_to_files_txt(directory.clone(), file.clone());
            let chunks = process_file(file.clone());
            add_to_faiss_lookup(directory.clone(), chunks.len(), file.to_str().unwrap().to_string());
        }
    }
}

pub fn add_to_files_txt(directory: PathBuf, file: PathBuf) {
    let files_txt = directory.join(".vs").join("files.txt");

    if !files_txt.exists() {
        if let Err(e) = File::create(&files_txt) {
            eprintln!("Failed to create files.txt: {}", e);
        }
    }

    let mut text_file = OpenOptions::new().append(true).open(&files_txt).unwrap();
    text_file.write_all(file.to_str().unwrap().as_bytes()).unwrap();
    text_file.write_all(b"\n").unwrap();
}

pub fn add_to_faiss_lookup(directory: PathBuf, num_chunks: usize, file_name: String) {
    let faiss_lookup = directory.join(".vs").join("faiss_lookup.txt");
    let mut faiss_lookup = OpenOptions::new().append(true).open(&faiss_lookup).unwrap();
    faiss_lookup.write_all(file_name.as_bytes()).unwrap();
    faiss_lookup.write_all(b" ").unwrap();
    faiss_lookup.write_all(num_chunks.to_string().as_bytes()).unwrap();
    faiss_lookup.write_all(b"\n").unwrap();
}

pub fn get_files_txt(directory: PathBuf) -> Vec<String> {
    let files_txt = directory.join(".vs").join("files.txt");
    let mut files = Vec::new();
    if files_txt.exists() {
        let text_file = File::open(files_txt).unwrap();
        let mut text_file = BufReader::new(text_file);
        let mut line = String::new();
        while text_file.read_line(&mut line).unwrap() > 0 {
            files.push(line.trim().to_string());
            line.clear();
        }
    }
    files
}

pub fn get_files(directory: PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(directory).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            files.push(path);
        }
    }
    files
}

pub fn process_file(file: PathBuf) -> Vec<String> {
    if file.extension().unwrap_or_default() == "pdf" {
        let chunks = prepare_pdf(&file);
        println!("Chunks: {:?}", chunks);
        chunks
    } else {
        let text = fs::read_to_string(file).expect("Failed to read file");
        let chunks = chunk_text(&text);
        println!("Chunks: {:?}", chunks);
        chunks
    }
}

pub fn chunk_text(text: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();
    let mut current_length = 0;
    let max_length = 1000;

    for line in text.lines() {
        if current_length + line.len() > max_length {
            chunks.push(current_chunk);
            current_chunk = String::new();
            current_length = 0;
        }
        current_chunk.push_str(line);
        current_length += line.len();
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    chunks
}

pub fn prepare_pdf(pdf_path: &PathBuf) -> Vec<String> {
    let bytes = fs::read(pdf_path).unwrap();
    let text = pdf_extract::extract_text_from_mem(&bytes).unwrap();
    chunk_text(&text)
}   

pub fn get_chunk(directory: PathBuf, vector_index: usize) -> Option<String> {
    // Open the faiss_lookup.txt file
    let faiss_lookup_path = directory.join(".vs").join("faiss_lookup.txt");
    let file = File::open(faiss_lookup_path).ok()?;
    let reader = BufReader::new(file);

    // Build a list of (filename, chunk_count)
    let mut file_chunks: Vec<(String, usize)> = Vec::new();
    for line in reader.lines() {
        let line = line.unwrap();
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 2 {
            let filename = parts[0].to_string();
            if let Ok(chunk_count) = parts[1].parse::<usize>() {
                file_chunks.push((filename, chunk_count));
            }
        }
    }

    // Find which file and chunk this index corresponds to
    let mut idx = vector_index;
    for (filename, chunk_count) in file_chunks {
        if idx < chunk_count {
            // This is the file and chunk we want
            let file_path = PathBuf::from(&filename);
            let chunks = process_file(file_path);
            if idx < chunks.len() {
                return Some(chunks[idx].clone());
            } else {
                return None;
            }
        } else {
            idx -= chunk_count;
        }
    }
    eprintln!("Vector index out of range"); 
    None
}