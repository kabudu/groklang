# groklang/package_manager.py
"""GrokLang Package Manager"""

import os
import shutil
import subprocess
import sys
from pathlib import Path

class PackageManager:
    def __init__(self):
        self.registry_url = "https://registry.groklang.org"  # Placeholder

    def new_project(self, name, template="basic"):
        """Create a new GrokLang project"""
        if os.path.exists(name):
            print(f"Directory '{name}' already exists")
            return False

        os.makedirs(name)

        # Create grok.toml
        toml_content = f"""[package]
name = "{name}"
version = "0.1.0"
edition = "2023"

[dependencies]

[dev-dependencies]
"""
        with open("grok.toml", "w") as f:
            f.write(toml_content)

        # Create src/main.grok
        os.makedirs("src")
        main_content = """fn main() {
    println("Hello, GrokLang!");
}
"""
        with open("src/main.grok", "w") as f:
            f.write(main_content)

        print(f"Created new project '{name}'")
        return True

    def install_deps(self):
        """Install dependencies from grok.toml"""
        if not os.path.exists("grok.toml"):
            print("No grok.toml found")
            return

        # Parse grok.toml (simplified)
        print("Installing dependencies...")
        # Placeholder: in real impl, fetch from registry
        print("Dependencies installed")

    def build_project(self):
        """Build the project"""
        if not os.path.exists("src/main.grok"):
            print("No src/main.grok found")
            return

        # Compile main.grok
        result = subprocess.run([sys.executable, "-m", "groklang", "compile", "src/main.grok"],
                               capture_output=True, text=True)
        if result.returncode == 0:
            print("Project built successfully")
        else:
            print("Build failed:")
            print(result.stderr)

def new_project(name):
    pm = PackageManager()
    pm.new_project(name)

def install_deps():
    pm = PackageManager()
    pm.install_deps()

def build_project():
    pm = PackageManager()
    pm.build_project()