# tests/test_lsp_completeness.py
"""Tests for LSP Completeness"""

import pytest
from unittest.mock import AsyncMock, Mock
from groklang.language_server import GrokLangLanguageServer, completion, hover, definition, references

@pytest.mark.asyncio
async def test_completion():
    params = Mock()
    result = await completion(params)
    assert len(result.items) > 10  # Keywords + functions

@pytest.mark.asyncio
async def test_hover():
    params = Mock()
    result = await hover(params)
    assert "Keyword" in result.contents.value

@pytest.mark.asyncio
async def test_definition():
    params = Mock()
    params.text_document.uri = "file://test.grok"
    result = await definition(params)
    assert result.uri == "file://test.grok"

@pytest.mark.asyncio
async def test_references():
    params = Mock()
    params.text_document.uri = "file://test.grok"
    result = await references(params)
    assert len(result) == 1