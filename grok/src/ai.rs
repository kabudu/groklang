// grok/src/ai.rs
//
// GrokLang AI Integration Module
// Supports multiple LLM providers: OpenAI, DeepSeek, and mock mode

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

// ============================================================================
// Configuration
// ============================================================================

#[derive(Debug, Clone)]
pub enum AiProvider {
    OpenAI,
    DeepSeek,
    Mock,
}

#[derive(Debug, Clone)]
pub struct AiConfig {
    pub provider: AiProvider,
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: String,
    pub timeout_secs: u64,
    pub max_tokens: u32,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: AiProvider::Mock,
            api_key: None,
            model: "gpt-3.5-turbo".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            timeout_secs: 30,
            max_tokens: 2048,
        }
    }
}

impl AiConfig {
    pub fn openai(api_key: String) -> Self {
        Self {
            provider: AiProvider::OpenAI,
            api_key: Some(api_key),
            model: "gpt-3.5-turbo".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            timeout_secs: 30,
            max_tokens: 2048,
        }
    }
    
    pub fn deepseek(api_key: String) -> Self {
        Self {
            provider: AiProvider::DeepSeek,
            api_key: Some(api_key),
            model: "deepseek-chat".to_string(),
            base_url: "https://api.deepseek.com".to_string(),
            timeout_secs: 60,
            max_tokens: 4096,
        }
    }
    
    pub fn mock() -> Self {
        Self {
            provider: AiProvider::Mock,
            api_key: None,
            model: "mock".to_string(),
            base_url: "".to_string(),
            timeout_secs: 0,
            max_tokens: 0,
        }
    }
    
    /// Load configuration from environment variables
    /// 
    /// Environment variables:
    /// - GROK_AI_PROVIDER: "openai", "deepseek", or "mock" (default: "mock")
    /// - GROK_AI_KEY: API key for the provider
    /// - GROK_AI_MODEL: Model name (optional, uses provider default)
    /// - GROK_AI_BASE_URL: Base URL (optional, uses provider default)
    pub fn from_env() -> Self {
        let provider = std::env::var("GROK_AI_PROVIDER")
            .unwrap_or_else(|_| "mock".to_string())
            .to_lowercase();
        
        let api_key = std::env::var("GROK_AI_KEY").ok();
        
        let mut config = match provider.as_str() {
            "openai" => {
                if let Some(key) = api_key {
                    Self::openai(key)
                } else {
                    eprintln!("Warning: GROK_AI_KEY not set for OpenAI, falling back to mock");
                    Self::mock()
                }
            }
            "deepseek" => {
                if let Some(key) = api_key {
                    Self::deepseek(key)
                } else {
                    eprintln!("Warning: GROK_AI_KEY not set for DeepSeek, falling back to mock");
                    Self::mock()
                }
            }
            _ => Self::mock(),
        };
        
        // Override model if specified
        if let Ok(model) = std::env::var("GROK_AI_MODEL") {
            config.model = model;
        }
        
        // Override base URL if specified
        if let Ok(base_url) = std::env::var("GROK_AI_BASE_URL") {
            config.base_url = base_url;
        }
        
        config
    }
}

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Serialize, Deserialize, Debug)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AiResponse {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub tokens_used: Option<u32>,
    pub latency_ms: u64,
    pub cached: bool,
}

// ============================================================================
// AI Operations
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum AiOperation {
    Optimize,           // Optimize code for performance
    Explain,            // Explain what code does
    Debug,              // Find bugs in code
    Refactor,           // Refactor code for readability
    GenerateTests,      // Generate test cases
    SecurityAudit,      // Check for security issues
    DocumentCode,       // Add documentation comments
    TranslateCode,      // Translate between languages
    Custom(String),     // Custom operation
}

impl AiOperation {
    pub fn system_prompt(&self) -> String {
        match self {
            AiOperation::Optimize => 
                "You are GrokLang's AI optimizer. Analyze the code and provide an optimized version with performance improvements. Return only the optimized code.".to_string(),
            AiOperation::Explain => 
                "You are GrokLang's AI explainer. Explain what the following code does in clear, concise terms. Include the purpose, inputs, outputs, and key logic.".to_string(),
            AiOperation::Debug => 
                "You are GrokLang's AI debugger. Analyze the code for bugs, logical errors, and potential issues. Provide specific fixes with explanations.".to_string(),
            AiOperation::Refactor => 
                "You are GrokLang's AI refactorer. Improve the code's readability, structure, and maintainability without changing its behavior. Return the refactored code.".to_string(),
            AiOperation::GenerateTests => 
                "You are GrokLang's AI test generator. Create comprehensive test cases for the given code, covering edge cases and typical usage scenarios.".to_string(),
            AiOperation::SecurityAudit => 
                "You are GrokLang's AI security auditor. Analyze the code for security vulnerabilities, unsafe patterns, and potential exploits. Provide remediation suggestions.".to_string(),
            AiOperation::DocumentCode => 
                "You are GrokLang's AI documenter. Add comprehensive documentation comments to the code including function descriptions, parameters, return values, and examples.".to_string(),
            AiOperation::TranslateCode => 
                "You are GrokLang's AI translator. Translate the code between programming languages while preserving functionality and idiomatic patterns.".to_string(),
            AiOperation::Custom(prompt) => prompt.clone(),
        }
    }
    
    pub fn name(&self) -> &str {
        match self {
            AiOperation::Optimize => "optimize",
            AiOperation::Explain => "explain",
            AiOperation::Debug => "debug",
            AiOperation::Refactor => "refactor",
            AiOperation::GenerateTests => "generate_tests",
            AiOperation::SecurityAudit => "security_audit",
            AiOperation::DocumentCode => "document_code",
            AiOperation::TranslateCode => "translate_code",
            AiOperation::Custom(_) => "custom",
        }
    }
}

// ============================================================================
// AI Trace (for debugging and benchmarking)
// ============================================================================

#[derive(Debug, Clone)]
pub struct AiTrace {
    pub operation: String,
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub latency_ms: u64,
    pub cached: bool,
    pub provider: String,
    pub model: String,
    pub success: bool,
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug, Default)]
pub struct AiTraceLog {
    pub traces: Vec<AiTrace>,
}

impl AiTraceLog {
    pub fn new() -> Self {
        Self { traces: Vec::new() }
    }
    
    pub fn add(&mut self, trace: AiTrace) {
        self.traces.push(trace);
    }
    
    pub fn summary(&self) -> String {
        if self.traces.is_empty() {
            return "No AI operations recorded.".to_string();
        }
        
        let total_ops = self.traces.len();
        let successful = self.traces.iter().filter(|t| t.success).count();
        let cached = self.traces.iter().filter(|t| t.cached).count();
        let total_latency: u64 = self.traces.iter().map(|t| t.latency_ms).sum();
        let avg_latency = total_latency / total_ops as u64;
        
        format!(
            "AI Trace Summary:\n\
             - Total Operations: {}\n\
             - Successful: {} ({:.1}%)\n\
             - Cached: {} ({:.1}%)\n\
             - Total Latency: {}ms\n\
             - Average Latency: {}ms",
            total_ops,
            successful, (successful as f64 / total_ops as f64) * 100.0,
            cached, (cached as f64 / total_ops as f64) * 100.0,
            total_latency,
            avg_latency
        )
    }
    
    pub fn detailed_report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== AI Operation Trace Log ===\n\n");
        
        for (i, trace) in self.traces.iter().enumerate() {
            report.push_str(&format!(
                "Operation #{}\n\
                 - Type: {}\n\
                 - Provider: {} ({})\n\
                 - Latency: {}ms\n\
                 - Cached: {}\n\
                 - Success: {}\n\
                 - Input Tokens: ~{}\n\
                 - Output Tokens: ~{}\n\n",
                i + 1,
                trace.operation,
                trace.provider,
                trace.model,
                trace.latency_ms,
                trace.cached,
                trace.success,
                trace.input_tokens,
                trace.output_tokens
            ));
        }
        
        report.push_str(&self.summary());
        report
    }
}

// ============================================================================
// AI Service
// ============================================================================

pub struct AiService {
    client: Client,
    config: AiConfig,
    cache: HashMap<String, AiResponse>,
    pub trace_log: AiTraceLog,
}

impl AiService {
    pub fn new() -> Self {
        Self::with_config(AiConfig::from_env())
    }
    
    pub fn with_config(config: AiConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs.max(10)))
            .build()
            .unwrap_or_else(|_| Client::new());
        
        Self {
            client,
            config,
            cache: HashMap::new(),
            trace_log: AiTraceLog::new(),
        }
    }
    
    pub fn config(&self) -> &AiConfig {
        &self.config
    }
    
    /// Process code with an AI operation
    pub async fn process(&mut self, operation: AiOperation, code: &str) -> Result<String, String> {
        let start = Instant::now();
        let cache_key = format!("{}:{}", operation.name(), code);
        
        // Check cache
        if let Some(cached_response) = self.cache.get(&cache_key) {
            let trace = AiTrace {
                operation: operation.name().to_string(),
                input_tokens: code.len() / 4,
                output_tokens: cached_response.output.as_ref().map(|s| s.len() / 4).unwrap_or(0),
                latency_ms: start.elapsed().as_millis() as u64,
                cached: true,
                provider: format!("{:?}", self.config.provider),
                model: self.config.model.clone(),
                success: cached_response.success,
                timestamp: std::time::SystemTime::now(),
            };
            self.trace_log.add(trace);
            
            if cached_response.success {
                let output = cached_response.output.as_ref().unwrap();
                if self.is_safe(output) {
                    return Ok(output.clone());
                } else {
                    return Err("AI output failed security check".to_string());
                }
            } else {
                return Err(cached_response.error.clone().unwrap_or_else(|| "Unknown error".to_string()));
            }
        }
        
        // Perform AI call
        let response = match self.config.provider {
            AiProvider::Mock => self.mock_process(&operation, code).await,
            AiProvider::OpenAI | AiProvider::DeepSeek => {
                self.llm_process(&operation, code).await
            }
        };
        
        let latency_ms = start.elapsed().as_millis() as u64;
        
        let ai_response = match &response {
            Ok(output) => AiResponse {
                success: true,
                output: Some(output.clone()),
                error: None,
                tokens_used: Some((code.len() + output.len()) as u32 / 4),
                latency_ms,
                cached: false,
            },
            Err(e) => AiResponse {
                success: false,
                output: None,
                error: Some(e.clone()),
                tokens_used: None,
                latency_ms,
                cached: false,
            },
        };
        
        let trace = AiTrace {
            operation: operation.name().to_string(),
            input_tokens: code.len() / 4,
            output_tokens: ai_response.output.as_ref().map(|s| s.len() / 4).unwrap_or(0),
            latency_ms,
            cached: false,
            provider: format!("{:?}", self.config.provider),
            model: self.config.model.clone(),
            success: ai_response.success,
            timestamp: std::time::SystemTime::now(),
        };
        self.trace_log.add(trace);
        
        self.cache.insert(cache_key, ai_response);
        
        match response {
            Ok(output) => {
                if self.is_safe(&output) {
                    Ok(output)
                } else {
                    Err("AI output failed security check".to_string())
                }
            }
            Err(e) => Err(e),
        }
    }
    
    async fn mock_process(&self, operation: &AiOperation, code: &str) -> Result<String, String> {
        // Simulate processing delay
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        match operation {
            AiOperation::Optimize => Ok(format!(
                "// Optimized version\n{}",
                code.replace("  ", " ")
            )),
            AiOperation::Explain => Ok(format!(
                "This code performs the following:\n\
                 1. Defines a function or process\n\
                 2. Processes input data\n\
                 3. Returns a computed result\n\n\
                 Original code:\n{}",
                code
            )),
            AiOperation::Debug => Ok(format!(
                "Debug analysis:\n\
                 - No obvious bugs detected\n\
                 - Consider adding error handling\n\
                 - Code appears syntactically correct\n\n\
                 Original code:\n{}",
                code
            )),
            AiOperation::Refactor => Ok(format!(
                "// Refactored for clarity\n{}",
                code
            )),
            AiOperation::GenerateTests => Ok(format!(
                "// Generated test cases\n\
                 #[test]\n\
                 fn test_basic() {{\n\
                     // Test with normal input\n\
                     assert!(true);\n\
                 }}\n\n\
                 #[test]\n\
                 fn test_edge_case() {{\n\
                     // Test with edge case\n\
                     assert!(true);\n\
                 }}"
            )),
            AiOperation::SecurityAudit => Ok(format!(
                "Security Audit Report:\n\
                 - No SQL injection vulnerabilities detected\n\
                 - No hardcoded credentials found\n\
                 - Consider input validation\n\n\
                 Analyzed code:\n{}",
                code
            )),
            AiOperation::DocumentCode => Ok(format!(
                "/// Function documentation\n\
                 /// \n\
                 /// # Arguments\n\
                 /// * Various inputs as defined\n\
                 /// \n\
                 /// # Returns\n\
                 /// The computed result\n\
                 {}", 
                code
            )),
            AiOperation::TranslateCode => Ok(format!(
                "// Translation to target language\n{}",
                code
            )),
            AiOperation::Custom(prompt) => Ok(format!(
                "// Custom operation: {}\n{}",
                prompt, code
            )),
        }
    }
    
    async fn llm_process(&self, operation: &AiOperation, code: &str) -> Result<String, String> {
        let api_key = self.config.api_key.as_ref()
            .ok_or("API key not configured")?;
        
        let endpoint = format!("{}/chat/completions", self.config.base_url);
        
        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: operation.system_prompt(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: format!("Code to process:\n\n```\n{}\n```", code),
                },
            ],
            max_tokens: self.config.max_tokens,
            temperature: 0.3,
        };
        
        let response = self.client
            .post(&endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("API error {}: {}", status, body));
        }
        
        let json: serde_json::Value = response.json().await
            .map_err(|e| format!("JSON parse error: {}", e))?;
        
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or("Invalid response format")?;
        
        Ok(content.to_string())
    }
    
    /// Security check for AI output
    pub fn is_safe(&self, output: &str) -> bool {
        let dangerous_patterns = [
            "eval(",
            "exec(",
            "system(",
            "os.popen",
            "__import__",
            "subprocess",
            "rm -rf",
            "DROP TABLE",
            "DELETE FROM",
            "; DROP",
        ];
        
        let lower_output = output.to_lowercase();
        !dangerous_patterns.iter().any(|p| lower_output.contains(&p.to_lowercase()))
    }
    
    /// Get trace log summary
    pub fn get_trace_summary(&self) -> String {
        self.trace_log.summary()
    }
    
    /// Get detailed trace report
    pub fn get_trace_report(&self) -> String {
        self.trace_log.detailed_report()
    }
    
    /// Clear cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ai_service_mock() {
        let mut service = AiService::with_config(AiConfig::mock());
        
        let result = service.process(AiOperation::Explain, "fn add(a, b) { a + b }").await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("This code performs"));
    }

    #[tokio::test]
    async fn test_ai_operations() {
        let mut service = AiService::with_config(AiConfig::mock());
        
        // Test all operations
        let code = "fn factorial(n) { if n <= 1 { 1 } else { n * factorial(n-1) } }";
        
        let ops = vec![
            AiOperation::Optimize,
            AiOperation::Explain,
            AiOperation::Debug,
            AiOperation::Refactor,
            AiOperation::GenerateTests,
            AiOperation::SecurityAudit,
            AiOperation::DocumentCode,
        ];
        
        for op in ops {
            let result = service.process(op.clone(), code).await;
            assert!(result.is_ok(), "Operation {:?} failed: {:?}", op, result.err());
        }
    }

    #[tokio::test]
    async fn test_ai_caching() {
        let mut service = AiService::with_config(AiConfig::mock());
        
        let code = "fn test() { 42 }";
        
        // First call
        let result1 = service.process(AiOperation::Explain, code).await;
        assert!(result1.is_ok());
        
        // Second call (should be cached)
        let result2 = service.process(AiOperation::Explain, code).await;
        assert!(result2.is_ok());
        
        // Check trace log
        assert_eq!(service.trace_log.traces.len(), 2);
        assert!(!service.trace_log.traces[0].cached);
        assert!(service.trace_log.traces[1].cached);
    }

    #[test]
    fn test_ai_safety() {
        let service = AiService::with_config(AiConfig::mock());
        
        assert!(service.is_safe("safe code"));
        assert!(service.is_safe("normal function"));
        assert!(!service.is_safe("eval('dangerous')"));
        assert!(!service.is_safe("os.exec(cmd)"));
        assert!(!service.is_safe("rm -rf /"));
        assert!(!service.is_safe("DROP TABLE users"));
    }

    #[test]
    fn test_config_from_env() {
        // Test default (mock)
        std::env::remove_var("GROK_AI_PROVIDER");
        let config = AiConfig::from_env();
        assert!(matches!(config.provider, AiProvider::Mock));
    }
    
    #[test]
    fn test_trace_log() {
        let mut log = AiTraceLog::new();
        
        log.add(AiTrace {
            operation: "test".to_string(),
            input_tokens: 100,
            output_tokens: 50,
            latency_ms: 150,
            cached: false,
            provider: "Mock".to_string(),
            model: "mock".to_string(),
            success: true,
            timestamp: std::time::SystemTime::now(),
        });
        
        let summary = log.summary();
        assert!(summary.contains("Total Operations: 1"));
        assert!(summary.contains("Successful: 1"));
    }
}
