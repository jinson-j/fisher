use std::path::PathBuf;
use std::fs::{File, OpenOptions, read_to_string, read};
use std::io::{Write, BufRead, BufReader};
use crate::faiss::VectorStore;
use crate::model::generate_embedding_document;



pub async fn setup_vector_store(directory: PathBuf) {
    let vs_dir = directory.join(".vs");

    if !vs_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&vs_dir) {
            eprintln!("Failed to create .vs directory: {}", e);
        }
    } 

    let files = get_files(directory.clone());
    // Read all file names from faiss_lookup.txt
    let faiss_lookup_path = directory.join(".vs").join("faiss_lookup.txt");
    let mut existing_files = Vec::new();
    let mut is_empty = true;
    if faiss_lookup_path.exists() {
        let file = File::open(&faiss_lookup_path).unwrap();
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap();
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 2 {
                existing_files.push(parts[0].to_string());
                is_empty = false;
            }
        }
    }

    // Assume embedding dimension is 768 for now
    let embedding_dim = 3072;
    let mut vector_store = VectorStore::new(embedding_dim).expect("Failed to create VectorStore");

    if is_empty {
        // Create new vector store and add all files
        for file in &files {
            let file_str = file.to_str().unwrap().to_string();
            let chunks = process_file(file.clone());
            if !chunks.is_empty() {
                let embeddings = generate_embedding_document(&chunks).await.expect("Failed to embed");
                vector_store.add(&embeddings).expect("Failed to add to vector store");
                add_to_faiss_lookup(directory.clone(), chunks.len(), file_str);
            }
        }
    } else {
        // Only add new files
        for file in &files {
            let file_str = file.to_str().unwrap().to_string();
            if !existing_files.contains(&file_str) {
                let chunks = process_file(file.clone());
                if !chunks.is_empty() {
                    let embeddings = generate_embedding_document(&chunks).await.expect("Failed to embed");
                    vector_store.add(&embeddings).expect("Failed to add to vector store");
                    add_to_faiss_lookup(directory.clone(), chunks.len(), file_str);
                }
            }
        }
    }
}

pub fn add_to_faiss_lookup(directory: PathBuf, num_chunks: usize, file_name: String) {
    let faiss_lookup = directory.join(".vs").join("faiss_lookup.txt");
    let mut faiss_lookup = OpenOptions::new().append(true).open(&faiss_lookup).unwrap();
    faiss_lookup.write_all(file_name.as_bytes()).unwrap();
    faiss_lookup.write_all(b" ").unwrap();
    faiss_lookup.write_all(num_chunks.to_string().as_bytes()).unwrap();
    faiss_lookup.write_all(b"\n").unwrap();
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
        let text = read_to_string(file).expect("Failed to read file");
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
    let bytes = read(pdf_path).unwrap();
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

/// Query the vector store with a string and return the indices of the nearest neighbors.
pub async fn query_vector_store(query: &str, directory: PathBuf) -> Result<Vec<usize>, Box<dyn std::error::Error + Send + Sync>> {
    use crate::faiss::VectorStore;
    use crate::model::generate_embedding_query;
    // Assume embedding dimension is 3072
    let embedding_dim = 3072;
    let mut vector_store = VectorStore::new(embedding_dim)?;

    // TODO: Load vectors from disk or reconstruct from files/faiss_lookup if persistence is needed
    // For now, this is a fresh index and will return nothing meaningful unless populated in this session

    // Generate embedding for the query string
    let embedding = generate_embedding_query(query).await?;
    // Query the vector store for the top 5 nearest neighbors
    let k = 5;
    let (_distances, indices) = vector_store.query(&embedding, k)?;
    Ok(indices.into_iter().map(|i| i.get().unwrap() as usize).collect())
}

