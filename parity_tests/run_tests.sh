#!/bin/bash
~/.cargo/bin/cargo build -p meridian_cli > /dev/null 2>&1
CLI="../target/debug/merid"
for f in *.mr; do
    echo "Testing $f..."
    VM_OUT=$($CLI run "$f")
    $CLI build "$f" > /dev/null 2>&1
    BIN="${f%.mr}"
    AOT_OUT=$(./"$BIN" || true)
    
    # Strip any warnings like ld: warning from AOT_OUT if it prints to stderr
    # actually AOT_OUT only captures stdout.
    
    if [ "$VM_OUT" = "$AOT_OUT" ]; then
        echo "PASS: $f"
    else
        echo "FAIL: $f (VM: $VM_OUT, AOT: $AOT_OUT)"
        exit 1
    fi
done
echo "All parity tests passed."
