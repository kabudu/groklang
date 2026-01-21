// grok/src/lsp.rs

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "GrokLang LSP server initialized!")
            .await;
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        let completions = vec![
            CompletionItem::new_simple("fn".to_string(), "function".to_string()),
            CompletionItem::new_simple("let".to_string(), "variable".to_string()),
        ];
        Ok(Some(CompletionResponse::Array(completions)))
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

pub async fn run_lsp() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}