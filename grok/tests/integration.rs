#[cfg(test)]
mod tests {
    use std::fs;
    use std::process::Command;

    #[test]
    fn test_compile_integration() {
        // Create a test file
        let code = "fn main() {}";
        fs::write("test_integration.grok", code).unwrap();

        // Run the compiler
        let output = Command::new("cargo")
            .args(&["run", "--release", "--", "compile", "test_integration.grok"])
            .output()
            .expect("Failed to run command");

        assert!(output.status.success());

        // Clean up
        fs::remove_file("test_integration.grok").unwrap();
    }
}
