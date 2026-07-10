#!/bin/bash
MERID="./target/release/merid"

echo "=== A1 Parser ==="
$MERID check audit_tests/a1_parser.mer 2>&1

echo "=== A2 Int Arith ==="
$MERID check audit_tests/a2_int_arith.mer 2>&1

echo "=== A3 Int/Number Coercion ==="
$MERID check audit_tests/a3_int_number.mer 2>&1

cat << 'INNER_EOF' > audit_tests/a4_array_assign.mer
fn main() {
    let mut arr = [1.0, 2.0, 3.0];
    arr[1] = 50.0;
}
main();
INNER_EOF
echo "=== A4 Array Assign ==="
$MERID check audit_tests/a4_array_assign.mer 2>&1

cat << 'INNER_EOF' > audit_tests/a5_struct_method.mer
struct Point { x: Number, y: Number }
fn main() {
    let p = Point { x: 3.0, y: 4.0 };
    let d = p.distance();
}
main();
INNER_EOF
echo "=== A5 Struct Method Check ==="
$MERID check audit_tests/a5_struct_method.mer 2>&1
echo "=== A5 Struct Method Run ==="
$MERID run audit_tests/a5_struct_method.mer 2>&1

cat << 'INNER_EOF' > audit_tests/a6_generics.mer
fn identity<T>(x: T) -> T { x }
fn main() {
    print identity(1);
}
main();
INNER_EOF
echo "=== A6 Generics ==="
$MERID check audit_tests/a6_generics.mer 2>&1

cat << 'INNER_EOF' > audit_tests/a9_borrow_alias.mer
fn consume(x: Number) -> Number { x }
fn main() {
    let mut x = 5.0;
    let r = &mut x;
    let y = consume(x);
    print *r;
}
main();
INNER_EOF
echo "=== A9 Borrow Checker ==="
$MERID check audit_tests/a9_borrow_alias.mer 2>&1

cat << 'INNER_EOF' > audit_tests/a10_main_entry.mer
fn main() {
    print 42;
}
INNER_EOF
echo "=== A10 Main Entry ==="
$MERID run audit_tests/a10_main_entry.mer 2>&1

