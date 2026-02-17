use crate::borrow_checker::BorrowChecker;
use crate::parser::Parser;
use crate::type_checker::TypeChecker;
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
    fn parse_line_col(message: &str) -> Option<Position> {
        let line_marker = "line ";
        let col_marker = " col ";

        let line_start = message.find(line_marker)? + line_marker.len();
        let line_digits: String = message[line_start..]
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        let line: u32 = line_digits.parse().ok()?;

        let col_start = message.find(col_marker)? + col_marker.len();
        let col_digits: String = message[col_start..]
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        let col: u32 = col_digits.parse().ok()?;

        Some(Position::new(line.saturating_sub(1), col.saturating_sub(1)))
    }

    async fn validate_document(&self, uri: Url, text: &str) {
        let mut diagnostics = Vec::new();
        match self.parser.parse_detailed(text) {
            Ok(ast) => {
                let mut type_checker = TypeChecker::new();
                if let Err(e) = type_checker.check(&ast) {
                    let start = Self::parse_line_col(&e).unwrap_or(Position::new(0, 0));
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start,
                            end: Position::new(start.line, start.character.saturating_add(1)),
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: None,
                        code_description: None,
                        source: Some("grok-type".to_string()),
                        message: e,
                        related_information: None,
                        tags: None,
                        data: None,
                    });
                }

                let mut borrow_checker = BorrowChecker::new();
                if let Err(e) = borrow_checker.check(&ast) {
                    let start = Self::parse_line_col(&e).unwrap_or(Position::new(0, 0));
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start,
                            end: Position::new(start.line, start.character.saturating_add(1)),
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: None,
                        code_description: None,
                        source: Some("grok-borrow".to_string()),
                        message: e,
                        related_information: None,
                        tags: None,
                        data: None,
                    });
                }
            }
            Err(e) => {
                let start = Position::new(
                    e.line.saturating_sub(1) as u32,
                    e.col.saturating_sub(1) as u32,
                );
                let diagnostic = Diagnostic {
                    range: Range {
                        start,
                        end: Position::new(start.line, start.character.saturating_add(1)),
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("grok-lsp".to_string()),
                    message: e.to_string(),
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
