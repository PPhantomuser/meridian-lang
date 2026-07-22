use std::process::Command;
use std::fs;

#[test]
fn test_fmt_basic() {
    let unformatted = "let  x:  Number= 10;
fn add( a:Number,b:Number)-> Number{a+b}
print  add(x, 20);";

    let temp_file = std::env::temp_dir().join("messy.mer");
    fs::write(&temp_file, unformatted).unwrap();

    let output = Command::new("cargo")
        .current_dir(std::env::current_dir().unwrap().parent().unwrap().parent().unwrap()) // point to root
        .args(["run", "--bin", "meridian", "--", "fmt", temp_file.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "Command failed: {}", String::from_utf8_lossy(&output.stderr));
    
    let formatted = fs::read_to_string(&temp_file).unwrap();
    let expected = "let x: Number = 10;

fn add(a: Number, b: Number) -> Number {
    a + b;
}

print add(x, 20);
";
    
    assert_eq!(formatted, expected);
}
