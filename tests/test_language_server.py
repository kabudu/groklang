# tests/test_language_server.py
"""Tests for GrokLang Language Server"""

import pytest
from unittest.mock import Mock, patch
from groklang.language_server import GrokLangLanguageServer, validate_document, create_diagnostic
from lsprotocol.types import DiagnosticSeverity, Position, Range

class TestLanguageServer:
    def test_create_diagnostic(self):
        diag = create_diagnostic("Test error", DiagnosticSeverity.Error)
        assert diag.message == "Test error"
        assert diag.severity == DiagnosticSeverity.Error
        assert diag.range.start.line == 0

    @patch('groklang.language_server.parser_mod.parser.parse')
    @patch('groklang.language_server.parser_mod.parser.errors', [])
    @patch('groklang.language_server.type_checker.check')
    @patch('groklang.language_server.type_checker.errors', [])
    def test_validate_document_success(self, mock_check, mock_errors, mock_parse, mock_parser_errors):
        mock_parse.return_value = Mock()
        doc = Mock()
        doc.source = "fn main() {}"
        doc.uri = "file://test.grok"

        server = GrokLangLanguageServer()
        with patch.object(server, 'publish_diagnostics') as mock_publish:
            import asyncio
            asyncio.run(validate_document(doc))
            mock_publish.assert_called_with(doc.uri, [])

    @patch('groklang.language_server.parser_mod.parser.parse')
    @patch('groklang.language_server.parser_mod.parser.errors', ["Parse error"])
    def test_validate_document_parse_error(self, mock_errors, mock_parse):
        mock_parse.return_value = None
        doc = Mock()
        doc.source = "invalid code"
        doc.uri = "file://test.grok"

        server = GrokLangLanguageServer()
        with patch.object(server, 'publish_diagnostics') as mock_publish:
            import asyncio
            asyncio.run(validate_document(doc))
            assert len(mock_publish.call_args[0][1]) == 1