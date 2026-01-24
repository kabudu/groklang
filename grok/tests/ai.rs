#[cfg(test)]
mod tests {
    use grok::ai::{AiConfig, AiOperation, AiService};

    #[tokio::test]
    async fn test_ai_service() {
        let mut service = AiService::with_config(AiConfig::mock());
        let result = service.process(AiOperation::Optimize, "fn add(a, b) { a + b }").await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Optimized"));
    }

    #[test]
    fn test_ai_safety() {
        let service = AiService::with_config(AiConfig::mock());
        assert!(service.is_safe("safe code"));
        assert!(!service.is_safe("eval('bad')"));
    }
}
