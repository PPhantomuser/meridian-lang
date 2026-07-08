use std::process::Command;
use std::fs;

#[test]
fn test_fmt_basic() {
    let unformatted = "let  x:  Number= 10;
fn add( a:Number,b:Number)-> Number{a+b}
print  add(x, 20);";

    fs::write("../examples/messy.mer", unformatted).unwrap();

    let output = Command::new("cargo")
        .current_dir("..")
        .args(["run", "-p", "meridian_cli", "--", "fmt", "examples/messy.mer"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "Command failed: {}", String::from_utf8_lossy(&output.stderr));
    
    let formatted = fs::read_to_string("../examples/messy.mer").unwrap();
    let expected = "let x: Number = 10;

fn add(a: Number, b: Number) -> Number {
    a + b;
}

print add(x, 20);
";
    
    assert_eq!(formatted, expected);
}
