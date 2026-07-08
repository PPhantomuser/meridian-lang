use std::process::Command;
use std::fs;

#[test]
fn test_stdlib_read_file() {
    // We expect the program to read its own source code
    let source = "let content = read_file(\"examples/read_self.mer\");
print content;";

    fs::write("../examples/read_self.mer", source).unwrap();

    let output = Command::new("cargo")
        .current_dir("..")
        .args(["run", "-p", "meridian_cli", "--", "run", "examples/read_self.mer"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "Command failed: {}", String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("let content = read_file"));
}
