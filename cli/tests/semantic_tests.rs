use std::process::Command;

#[test]
fn test_immutable_error() {
    let output = Command::new("cargo")
        .current_dir("..")
        .args(["run", "-p", "meridian_cli", "--", "check", "examples/immutable_error.mer"])
        .output()
        .expect("Failed to execute command");
    
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("MER0101")); // Immutable assignment error
}

#[test]
fn test_type_error() {
    let output = Command::new("cargo")
        .current_dir("..")
        .args(["run", "-p", "meridian_cli", "--", "check", "examples/type_error.mer"])
        .output()
        .expect("Failed to execute command");
    
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("MER0102")); // Type mismatch error
}

#[test]
fn test_valid_semantics() {
    let output = Command::new("cargo")
        .current_dir("..")
        .args(["run", "-p", "meridian_cli", "--", "check", "examples/valid_semantics.mer"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "Command failed: {}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_func_arg_error() {
    let output = Command::new("cargo")
        .current_dir("..")
        .args(["run", "-p", "meridian_cli", "--", "check", "examples/func_arg_error.mer"])
        .output()
        .expect("Failed to execute command");
    
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("MER0110")); // Argument count mismatch
}

#[test]
fn test_func_return_error() {
    let output = Command::new("cargo")
        .current_dir("..")
        .args(["run", "-p", "meridian_cli", "--", "check", "examples/func_return_error.mer"])
        .output()
        .expect("Failed to execute command");
    
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("MER0111")); // Invalid return type
}

#[test]
fn test_fibonacci() {
    let output = Command::new("cargo")
        .current_dir("..")
        .args(["run", "-p", "meridian_cli", "--", "run", "examples/fibonacci.mer"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "Command failed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).unwrap();
}

#[test]
fn test_macros() {
    let output = Command::new("cargo")
        .current_dir("..")
        .args(["run", "-p", "meridian_cli", "--", "run", "examples/macros.mer"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Hello from macro!"));
    assert!(stdout.contains("30"));
}

#[test]
fn test_modules() {
    let output = Command::new("cargo")
        .current_dir("..")
        .args(["run", "-p", "meridian_cli", "--", "run", "examples/main.mer"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "Command failed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("30"));
}

#[test]
fn test_cyclic_imports() {
    let output = Command::new("cargo")
        .current_dir("..")
        .args(["run", "-p", "meridian_cli", "--", "check", "examples/cycle_a.mer"])
        .output()
        .expect("Failed to execute command");
    
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("MER0200")); // Cyclic dependency detected
}

#[test]
fn test_borrow_mut_immut() {
    let output = Command::new("cargo")
        .current_dir("..")
        .args(["run", "-p", "meridian_cli", "--", "check", "examples/borrow_mut_immut.mer"])
        .output()
        .expect("Failed to execute command");
    
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("MER0121")); // Cannot borrow immutable mutably
}

#[test]
fn test_borrow_mut_mut() {
    let output = Command::new("cargo")
        .current_dir("..")
        .args(["run", "-p", "meridian_cli", "--", "check", "examples/borrow_mut_mut.mer"])
        .output()
        .expect("Failed to execute command");
    
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("MER0120")); // Cannot borrow mutably because it is already borrowed
}

#[test]
fn test_borrow_elision_fail() {
    let output = Command::new("cargo")
        .current_dir("..")
        .args(["run", "-p", "meridian_cli", "--", "check", "examples/borrow_elision_fail.mer"])
        .output()
        .expect("Failed to execute command");
    
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("MER0123")); // Lifetime elision fails
}

#[test]
#[ignore]
fn test_async_await() {
    let output = Command::new("cargo")
        .current_dir("..")
        .args(["run", "-p", "meridian_cli", "--", "run", "examples/async_await.mer"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "Command failed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Fetching..."));
    assert!(stdout.contains("10"));
    assert!(stdout.contains("20"));
    assert!(stdout.contains("Done!"));
}
