// grok/src/ai.rs

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct AiRequest {
    operation: String,
    input: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct AiResponse {
    success: bool,
    output: Option<String>,
    error: Option<String>,
}

pub struct AiService {
    client: Client,
    cache: HashMap<String, AiResponse>,
}

impl AiService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            cache: HashMap::new(),
        }
    }

    pub async fn process(&mut self, operation: &str, code: &str) -> Result<String, String> {
        let key = format!("{}:{}", operation, code);
        if let Some(resp) = self.cache.get(&key) {
            if resp.success {
                // Security check
                if self.is_safe(&resp.output.as_ref().unwrap()) {
                    return Ok(resp.output.as_ref().unwrap().clone());
                } else {
                    return Err("AI output failed security check".to_string());
                }
            } else {
                return Err(resp.error.as_ref().unwrap().clone());
            }
        }

        // Mock AI call for now
        let response = AiResponse {
            success: true,
            output: Some(format!("Processed {}: {}", operation, code)),
            error: None,
        };

        self.cache.insert(key, response.clone());
        if response.success {
            Ok(response.output.unwrap())
        } else {
            Err(response.error.unwrap())
        }
    }

    pub fn is_safe(&self, output: &str) -> bool {
        // Basic checks
        !output.contains("eval(") && !output.contains("exec(")
    }
}