use std::fs;
use std::process::Command;

pub struct PackageManager;

impl PackageManager {
    pub fn new() -> Self {
        Self
    }

    pub fn install(&self, deps: Vec<String>) -> Result<(), String> {
        for dep in deps {
            Command::new("cargo")
                .args(&["add", &dep])
                .status()
                .map_err(|e| format!("Failed to install {}: {}", dep, e))?;
        }
        Ok(())
    }

    pub fn build(&self) -> Result<(), String> {
        Command::new("cargo")
            .arg("build")
            .arg("--release")
            .status()
            .map_err(|e| format!("Build failed: {}", e))?;
        Ok(())
    }

    pub fn test(&self) -> Result<(), String> {
        Command::new("cargo")
            .arg("test")
            .status()
            .map_err(|e| format!("Tests failed: {}", e))?;
        Ok(())
    }
}
