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
                let output = resp.output.as_ref().unwrap();
                if self.is_safe(output) {
                    return Ok(output.clone());
                } else {
                    return Err("AI output failed security check".to_string());
                }
            } else {
                return Err(resp.error.as_ref().unwrap_or(&"Unknown error".to_string()).clone());
            }
        }

        let api_key = std::env::var("GROK_AI_KEY").ok();
        
        let response = if let Some(key) = api_key {
            // Real OpenAI call
            let mut messages = Vec::new();
            let mut system_msg = HashMap::new();
            system_msg.insert("role", "system");
            system_msg.insert("content", "You are the GrokLang AI agent. You perform code operations as requested.");
            messages.push(system_msg);
            
            let mut user_msg = HashMap::new();
            user_msg.insert("role", "user");
            let prompt = format!("Operation: {}\nCode:\n{}", operation, code);
            user_msg.insert("content", &prompt);
            messages.push(user_msg);
            
            // Note: In a real implementation we would use a more robust JSON structure
            // This is a simplified version for demonstration
            
            let res = self.client.post("https://api.openai.com/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", key))
                .json(&serde_json::json!({
                    "model": "gpt-3.5-turbo",
                    "messages": messages
                }))
                .send()
                .await
                .map_err(|e| e.to_string())?;

            if res.status().is_success() {
                let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
                let content = json["choices"][0]["message"]["content"].as_str()
                    .ok_or("Invalid response format")?;
                AiResponse {
                    success: true,
                    output: Some(content.to_string()),
                    error: None,
                }
            } else {
                AiResponse {
                    success: false,
                    output: None,
                    error: Some(format!("API error: {}", res.status())),
                }
            }
        } else {
            // Mock AI call
            AiResponse {
                success: true,
                output: Some(format!("Processed {}: {}", operation, code)),
                error: None,
            }
        };

        self.cache.insert(key, response.clone());
        if response.success {
            let output = response.output.unwrap();
            if self.is_safe(&output) {
                Ok(output)
            } else {
                Err("AI output failed security check".to_string())
            }
        } else {
            Err(response.error.unwrap())
        }
    }

    pub fn is_safe(&self, output: &str) -> bool {
        // Basic checks
        !output.contains("eval(") && !output.contains("exec(")
    }
}