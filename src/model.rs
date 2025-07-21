use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::chat_interface::Message;
use std::env;


// Chat/LLM Structures

#[derive(Debug, Serialize, Deserialize)]
struct Part {
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ContentWithRole {
    role: String,
    parts: Vec<Part>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GenerateContentRequest {
    contents: Vec<ContentWithRole>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GenerateContentResponse {
    candidates: Option<Vec<Candidate>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Candidate {
    content: ContentWithRole,
    finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SingleEmbeddingRequest {
    model: String,
    content: Content,
    #[serde(rename = "taskType")]
    task_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct BatchEmbeddingRequest {
    requests: Vec<SingleEmbeddingRequest>,
}

fn sender_to_role(sender: &str) -> &str {
    match sender {
        "User" => "user",
        "LLM" => "model",
        _ => "user", // fallback
    }
}
pub async fn generate_response(messages: &[Message]) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {

    let client = Client::new();
    let api_key = env::var("GEMINI_API_KEY")
        .map_err(|_| "GEMINI_API_KEY not set in environment")?;
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

    let contents: Vec<ContentWithRole> = messages.iter().map(|msg| ContentWithRole {
        role: sender_to_role(&msg.sender).to_string(),
        parts: vec![Part { text: msg.content.clone() }],
    }).collect();

    let request_body = GenerateContentRequest { contents };

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(format!("API request failed: {}", error_text).into());
    }

    let response_body: GenerateContentResponse = response.json().await?;
    if let Some(candidates) = response_body.candidates {
        if let Some(candidate) = candidates.first() {
            if let Some(part) = candidate.content.parts.first() {
                return Ok(part.text.clone());
            }
        }
    }
    Err("No response generated".into())
}

pub async fn generate_embedding_document(texts: &[String]) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();
    let api_key = env::var("GEMINI_API_KEY")
        .map_err(|_| "GEMINI_API_KEY not set in environment")?;
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-embedding-001:batchEmbedContents?key={}",
        api_key
    );

    let requests: Vec<SingleEmbeddingRequest> = texts.iter().map(|t| SingleEmbeddingRequest {
        model: "models/gemini-embedding-001".to_string(),
        content: Content {
            parts: vec![Part { text: t.clone() }],
        },
        task_type: "RETRIEVAL_DOCUMENT".to_string(),
    }).collect();

    let request_body = BatchEmbeddingRequest { requests };

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(format!("API request failed: {}", error_text).into());
    }

    let response_body: serde_json::Value = response.json().await?;
    let mut embeddings = Vec::new();
    if let Some(arr) = response_body.get("embeddings").and_then(|v| v.as_array()) {
        for emb in arr {
            if let Some(values) = emb.get("values").and_then(|v| v.as_array()) {
                let vec: Vec<f32> = values.iter().filter_map(|x| x.as_f64().map(|f| f as f32)).collect();
                embeddings.push(vec);
            }
        }
    }
    Ok(embeddings)
}

pub async fn generate_embedding_query(query: &str) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();
    let api_key = env::var("GEMINI_API_KEY")
        .map_err(|_| "GEMINI_API_KEY not set in environment")?;
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-embedding-001:embedContent?key={}",
        api_key
    );

    let request_body = serde_json::json!({
        "model": "models/gemini-embedding-001",
        "content": {
            "parts": [
                {"text": query}
            ]
        },
        "taskType": "RETRIEVAL_QUERY"
    });

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(format!("API request failed: {}", error_text).into());
    }

    let response_body: serde_json::Value = response.json().await?;
    if let Some(embedding) = response_body.get("embedding") {
        if let Some(values) = embedding.get("values").and_then(|v| v.as_array()) {
            let embedding: Vec<f32> = values.iter().filter_map(|x| x.as_f64().map(|f| f as f32)).collect();
            return Ok(embedding);
        }
    }
    Err("No embedding generated".into())
}


