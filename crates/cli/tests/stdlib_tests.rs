use std::process::Command;
use std::fs;

#[test]
fn test_stdlib_read_file() {
    // We expect the program to read its own source code
    let source = "let content = unsafe { read_file(\"examples/read_self.mer\") };
print content;";

    let examples_dir = std::env::current_dir().unwrap().parent().unwrap().parent().unwrap().join("examples");
    fs::write(examples_dir.join("read_self.mer"), source).unwrap();

    let output = Command::new("cargo")
        .current_dir(std::env::current_dir().unwrap().parent().unwrap().parent().unwrap())
        .args(["run", "--bin", "merid", "--", "run", "examples/read_self.mer", "--allow-fs"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "Command failed: {}", String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("let content = unsafe { read_file"));
}
