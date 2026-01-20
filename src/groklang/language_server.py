# groklang/language_server.py
"""Language Server for GrokLang IDE support using LSP"""

from pygls.server import LanguageServer
from pygls.protocol import InitializeResult, InitializeParams
from pygls.workspace import Document
from lsprotocol.types import (
    TEXT_DOCUMENT_DID_OPEN,
    TEXT_DOCUMENT_DID_CHANGE,
    TEXT_DOCUMENT_COMPLETION,
    TEXT_DOCUMENT_DIAGNOSTIC,
    CompletionParams,
    CompletionList,
    CompletionItem,
    CompletionItemKind,
    Diagnostic,
    DiagnosticSeverity,
    Position,
    Range,
)
from groklang import parser as parser_mod
from groklang import type_checker

KEYWORDS = [
    "fn", "let", "if", "else", "while", "for", "return", "struct", "enum",
    "pub", "mut", "const", "import", "as", "from", "match", "case", "async", "await",
    "actor", "supervise", "spawn", "send", "receive", "module", "macro", "test", "optimize"
]

class GrokLangLanguageServer(LanguageServer):
    def __init__(self):
        super().__init__("GrokLang Language Server", "0.1.0")

    async def initialize(self, params: InitializeParams) -> InitializeResult:
        result = InitializeResult(capabilities=self.server_capabilities)
        return result

server = GrokLangLanguageServer()

@server.feature(TEXT_DOCUMENT_DID_OPEN)
async def did_open(params):
    doc = server.workspace.get_document(params.text_document.uri)
    await validate_document(doc)

@server.feature(TEXT_DOCUMENT_DID_CHANGE)
async def did_change(params):
    doc = server.workspace.get_document(params.text_document.uri)
    await validate_document(doc)

async def validate_document(doc: Document):
    code = doc.source
    diagnostics = []

    # Parse
    ast = parser_mod.parser.parse(code)
    if parser_mod.parser.errors:
        for error in parser_mod.parser.errors:
            diagnostics.append(create_diagnostic(error, DiagnosticSeverity.Error))

    # Type check if no parse errors
    if ast and not parser_mod.parser.errors:
        try:
            type_checker.check(ast)
            if type_checker.errors:
                for error in type_checker.errors:
                    diagnostics.append(create_diagnostic(error, DiagnosticSeverity.Error))
        except Exception as e:
            diagnostics.append(create_diagnostic(str(e), DiagnosticSeverity.Error))

    server.publish_diagnostics(doc.uri, diagnostics)

def create_diagnostic(message: str, severity: DiagnosticSeverity) -> Diagnostic:
    # Parse error message to extract position if possible
    # For simplicity, assume position 0,0
    range_ = Range(start=Position(line=0, character=0), end=Position(line=0, character=1))
    return Diagnostic(range=range_, message=message, severity=severity)

@server.feature(TEXT_DOCUMENT_COMPLETION)
async def completion(params: CompletionParams) -> CompletionList:
    items = []
    for keyword in KEYWORDS:
        items.append(CompletionItem(label=keyword, kind=CompletionItemKind.Keyword))
    return CompletionList(is_incomplete=False, items=items)

if __name__ == "__main__":
    import asyncio
    asyncio.run(server.start_io())