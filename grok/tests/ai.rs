#[cfg(test)]
mod tests {
    use grok::ai::AiService;

    #[tokio::test]
    async fn test_ai_service() {
        let mut service = AiService::new();
        let result = service.process("optimize", "fn add(a, b) { a + b }").await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Processed optimize"));
    }

    #[test]
    fn test_ai_safety() {
        let service = AiService::new();
        assert!(service.is_safe("safe code"));
        assert!(!service.is_safe("eval('bad')"));
    }
}
