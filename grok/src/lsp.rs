use crate::parser::Parser;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
    parser: Parser,
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

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.validate_document(params.text_document.uri, &params.text_document.text)
            .await;
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.pop() {
            self.validate_document(params.text_document.uri, &change.text)
                .await;
        }
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        let completions = vec![
            CompletionItem::new_simple("fn".to_string(), "Define a function".to_string()),
            CompletionItem::new_simple("let".to_string(), "Bind a variable".to_string()),
            CompletionItem::new_simple("struct".to_string(), "Define a structure".to_string()),
            CompletionItem::new_simple("actor".to_string(), "Define an actor".to_string()),
            CompletionItem::new_simple("spawn".to_string(), "Spawn an actor".to_string()),
            CompletionItem::new_simple("match".to_string(), "Pattern matching".to_string()),
        ];
        Ok(Some(CompletionResponse::Array(completions)))
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

impl Backend {
    async fn validate_document(&self, uri: Url, text: &str) {
        let mut diagnostics = Vec::new();
        match self.parser.parse(text) {
            Ok(_) => {
                // No syntax errors
            }
            Err(e) => {
                // Simple error parsing. In a real implementation we'd parse the error string
                // or have the parser return structured errors with line/col.
                // For now, let's just report the error on the first line.
                let diagnostic = Diagnostic {
                    range: Range {
                        start: Position::new(0, 0),
                        end: Position::new(0, 1),
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("grok-lsp".to_string()),
                    message: e,
                    related_information: None,
                    tags: None,
                    data: None,
                };
                diagnostics.push(diagnostic);
            }
        }
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

pub async fn run_lsp() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        parser: Parser::new(),
    });
    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}
