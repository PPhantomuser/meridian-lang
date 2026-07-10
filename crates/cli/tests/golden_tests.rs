use std::process::Command;

#[test]
fn test_hello_ast() {
    let output = Command::new("cargo")
        .current_dir(std::env::current_dir().unwrap().parent().unwrap().parent().unwrap())
        .args(["run", "--bin", "merid", "--", "ast", "examples/hello.mer"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "Command failed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Print"));
}

