# RFC 0002: Generics, Traits, and Methods in Meridian

## 1. Introduction

As Meridian targets complex engineering and algorithmic tasks, it requires a robust system for abstraction and polymorphism. This RFC proposes a unified design for:
1. **Generics**: Type parameterization for structs, enums, and functions.
2. **Methods**: Struct and Enum specific functions (impl blocks).
3. **Traits (Interfaces)**: A mechanism for defining shared behavior and bounding generics.

The goal is to maintain Meridian's "Predictable Memory Model" and "AOT-first" capability while filling the single largest gap identified in the Round 4 and 5 audits.

## 2. Generics

### Syntax

Generics will use angle brackets `<T>`, heavily inspired by Rust and TypeScript.

```meridian
struct Box<T> {
    value: T
}

fn identity<T>(x: T) -> T {
    return x;
}
```

### Semantics and Implementation

To ensure Cranelift AOT compatibility without runtime type-erasure overhead, generics will be **monomorphized** at compile time. 
- The `semantic` crate will track instantiated types.
- The `ir` and `aot` crates will generate unique functions/structs for each specialized type (e.g., `identity_Int`, `identity_String`).

## 3. Methods (Impl Blocks)

To avoid polluting the global namespace and to group related behaviors, Meridian will introduce `impl` blocks.

### Syntax

```meridian
struct Rectangle {
    width: num,
    height: num
}

impl Rectangle {
    fn area(self) -> num {
        return self.width * self.height;
    }
}
```

### Semantics

Methods are simply syntactic sugar for static functions where the first argument is `self`.
- `rect.area()` is desugared to `Rectangle::area(rect)`.
- `self` will follow standard Meridian borrow-checking rules (by-value, or borrowed if referenced).

## 4. Traits (Interfaces)

Traits define shared behavior. They serve as bounds for generics to ensure type safety.

### Syntax

```meridian
trait Drawable {
    fn draw(self);
}

impl Drawable for Rectangle {
    fn draw(self) {
        print "Drawing rectangle";
    }
}

// Bounding a generic
fn render_scene<T: Drawable>(item: T) {
    item.draw();
}
```

### Semantics

In the MVP, Traits will **only** be used for static dispatch via Generic Bounds. 
- Dynamic dispatch (trait objects, e.g., `Box<dyn Drawable>`) will be deferred to a future RFC to keep the MVP scope constrained and predictable for AOT compilation.

## 5. Security and Agentic Considerations

- Generic monomorphization increases binary size but preserves static analysis transparency, which is crucial for the "Agentic by Design" philosophy.
- Trait bounds provide rigid guarantees about what an agent-generated generic function is allowed to do.

## 6. Next Steps

1. Add parsing support for `<T>`, `impl`, and `trait` keywords to `crates/parser`.
2. Add name-resolution and monomorphization logic to `crates/semantic`.
3. Add trait bound checking to `crates/semantic`.
4. Ensure AOT parity for all monomorphized outputs.
