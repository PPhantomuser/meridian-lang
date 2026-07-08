# Meridian V3.5 FFI Guide

This guide details the C Foreign Function Interface (FFI) introduced in Meridian V3.5.

## 1. Extern "C" Blocks
To call a C function, you must declare it inside an `extern "C"` block. These functions are strictly evaluated as foreign functions by the semantic analyzer.

```meridian
extern "C" {
    fn puts(s: *const u8) -> Number;
}
```

## 2. The `unsafe` Boundary
Meridian is memory-safe by default. Foreign function calls and raw pointer dereferencing bypass the semantic analyzer's safety guarantees and borrow checks. Therefore, calling any `extern` function **must** occur inside an `unsafe { ... }` block. Attempting to call them outside will result in a compile-time semantic error.

```meridian
fn say_hello() {
    unsafe {
        // Safe to call puts here
        puts("Hello, world!");
    }
}
```

## 3. C ABI Mapping & Raw Pointers
Meridian's high-level `String` type is an ARC-managed, bounds-checked object. It is **not** safe to pass directly to a C function expecting a null-terminated `char*`.

To interact with C, Meridian provides minimal raw pointer types specifically for FFI:
- `*const u8` for C strings.
- `*mut Number` for mutable data buffers.

When calling C functions, you must explicitly pass data in the layout C expects. For strings, this typically involves converting a Meridian `String` into a raw `*const u8` (which is guaranteed to be null-terminated in memory when passed via FFI).

## 4. VM Runtime Trap
The Meridian interpreter (`merid run` without `--release`) does **not** dynamically load C libraries (no `dlopen`/`libloading` in V3). If the VM attempts to execute an `extern "C"` function, it will throw a graceful runtime trap.

FFI execution is strictly supported via JIT/AOT (`merid run --release`) where the Cranelift backend generates standard C-ABI calls and links them safely.
