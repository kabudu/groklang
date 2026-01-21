#!/bin/bash
# Build GrokLang binary using PyInstaller

set -e

echo "Building GrokLang binary..."

# Activate virtual environment if it exists
if [ -d "venv" ]; then
    source venv/bin/activate
fi

# Install dependencies
pip install -r requirements.txt
pip install PyInstaller

# Build binary
pyinstaller --onefile --name grok src/groklang/__main__.py

echo "Binary created: dist/grok"
echo "You can now run: ./dist/grok hello.grok"