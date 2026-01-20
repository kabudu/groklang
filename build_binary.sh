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
pyinstaller --onefile --name groklang src/groklang/__main__.py

echo "Binary created: dist/groklang"
echo "You can now run: ./dist/groklang hello.grok"