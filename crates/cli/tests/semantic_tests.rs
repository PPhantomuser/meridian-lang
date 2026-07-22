#![allow(unused)]
use std::process::Command;

#[test]
fn test_immutable_error() {
    let output = Command::new("cargo")
        .current_dir(std::env::current_dir().unwrap().parent().unwrap().parent().unwrap())
        .args(["run", "--bin", "meridian", "--", "check", "examples/immutable_error.mer"])
        .output()
        .expect("Failed to execute command");
    
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("MER0101")); // Immutable assignment error
}

#[test]
fn test_type_error() {
    let output = Command::new("cargo")
        .current_dir(std::env::current_dir().unwrap().parent().unwrap().parent().unwrap())
        .args(["run", "--bin", "meridian", "--", "check", "examples/type_error.mer"])
        .output()
        .expect("Failed to execute command");
    
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("MER0102")); // Type mismatch error
}

#[test]
fn test_valid_semantics() {
    let output = Command::new("cargo")
        .current_dir(std::env::current_dir().unwrap().parent().unwrap().parent().unwrap())
        .args(["run", "--bin", "meridian", "--", "check", "examples/valid_semantics.mer"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "Command failed: {}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_func_arg_error() {
    let output = Command::new("cargo")
        .current_dir(std::env::current_dir().unwrap().parent().unwrap().parent().unwrap())
        .args(["run", "--bin", "meridian", "--", "check", "examples/func_arg_error.mer"])
        .output()
        .expect("Failed to execute command");
    
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("MER0110")); // Argument count mismatch
}

#[test]
fn test_func_return_error() {
    let output = Command::new("cargo")
        .current_dir(std::env::current_dir().unwrap().parent().unwrap().parent().unwrap())
        .args(["run", "--bin", "meridian", "--", "check", "examples/func_return_error.mer"])
        .output()
        .expect("Failed to execute command");
    
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("MER0111")); // Invalid return type
}

#[test]
fn test_fibonacci() {
    let output = Command::new("cargo")
        .current_dir(std::env::current_dir().unwrap().parent().unwrap().parent().unwrap())
        .args(["run", "--bin", "meridian", "--", "run", "examples/fibonacci.mer"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "Command failed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).unwrap();
}

#[test]
fn test_macros() {
    let output = Command::new("cargo")
        .current_dir(std::env::current_dir().unwrap().parent().unwrap().parent().unwrap())
        .args(["run", "--bin", "meridian", "--", "run", "examples/macros.mer"])
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
        .current_dir(std::env::current_dir().unwrap().parent().unwrap().parent().unwrap())
        .args(["run", "--bin", "meridian", "--", "run", "examples/main.mer"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "Command failed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("30"));
}

#[test]
fn test_cyclic_imports() {
    let output = Command::new("cargo")
        .current_dir(std::env::current_dir().unwrap().parent().unwrap().parent().unwrap())
        .args(["run", "--bin", "meridian", "--", "check", "examples/cycle_a.mer"])
        .output()
        .expect("Failed to execute command");
    
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("MER0200")); // Cyclic dependency detected
}

#[test]
fn test_borrow_mut_immut() {
    let output = Command::new("cargo")
        .current_dir(std::env::current_dir().unwrap().parent().unwrap().parent().unwrap())
        .args(["run", "--bin", "meridian", "--", "check", "examples/borrow_mut_immut.mer"])
        .output()
        .expect("Failed to execute command");
    
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("MER0121")); // Cannot borrow immutable mutably
}

#[test]
fn test_borrow_mut_mut() {
    let output = Command::new("cargo")
        .current_dir(std::env::current_dir().unwrap().parent().unwrap().parent().unwrap())
        .args(["run", "--bin", "meridian", "--", "check", "examples/borrow_mut_mut.mer"])
        .output()
        .expect("Failed to execute command");
    
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("MER0120")); // Cannot borrow mutably because it is already borrowed
}

#[test]
fn test_borrow_elision_fail() {
    let output = Command::new("cargo")
        .current_dir(std::env::current_dir().unwrap().parent().unwrap().parent().unwrap())
        .args(["run", "--bin", "meridian", "--", "check", "examples/borrow_elision_fail.mer"])
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
        .current_dir(std::env::current_dir().unwrap().parent().unwrap().parent().unwrap())
        .args(["run", "--bin", "meridian", "--", "run", "examples/async_await.mer"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "Command failed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Fetching..."));
    assert!(stdout.contains("10"));
    assert!(stdout.contains("20"));
    assert!(stdout.contains("Done!"));
}

#[test]
fn test_result_option_regression() {
    let output = Command::new("cargo")
        .current_dir(std::env::current_dir().unwrap().parent().unwrap().parent().unwrap())
        .args(["run", "--bin", "meridian", "--", "check", "examples/result_option_test.mer"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "Command failed: {}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_kitchen_sink() {
    let output = Command::new("cargo")
        .current_dir(std::env::current_dir().unwrap().parent().unwrap().parent().unwrap())
        .args(["run", "--bin", "meridian", "--", "run", "examples/kitchen_sink.mer"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "Command failed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Enum(Variant 0) { Point }"));
    assert!(stdout.contains("Enum(Variant 0) { Line }"));
}

#[test]
fn test_generic_trait_dispatch() {
    let output = Command::new("cargo")
        .current_dir(std::env::current_dir().unwrap().parent().unwrap().parent().unwrap())
        .args(["run", "--bin", "meridian", "--", "run", "test_dispatch.mer"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "Command failed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("12.566"));
    assert!(stdout.contains("9"));
}
