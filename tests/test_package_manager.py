# tests/test_package_manager.py
"""Tests for GrokLang Package Manager"""

import pytest
import os
import tempfile
from unittest.mock import patch
from groklang.package_manager import PackageManager

class TestPackageManager:
    def test_new_project(self):
        pm = PackageManager()
        with tempfile.TemporaryDirectory() as tmpdir:
            os.chdir(tmpdir)
            success = pm.new_project("test_project")
            assert success
            assert os.path.exists("test_project")
            assert os.path.exists("test_project/grok.toml")
            assert os.path.exists("test_project/src/main.grok")

    def test_install_deps_no_toml(self):
        pm = PackageManager()
        with tempfile.TemporaryDirectory() as tmpdir:
            os.chdir(tmpdir)
            pm.install_deps()  # Should not crash

    @patch('subprocess.run')
    def test_build_project(self, mock_run):
        mock_run.return_value.returncode = 0
        pm = PackageManager()
        with tempfile.TemporaryDirectory() as tmpdir:
            os.chdir(tmpdir)
            os.makedirs("src")
            with open("src/main.grok", "w") as f:
                f.write("fn main() {}")
            pm.build_project()
            mock_run.assert_called_once()