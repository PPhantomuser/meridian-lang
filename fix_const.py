import re

with open("crates/backend-cranelift/src/aot.rs", "r") as f:
    content = f.read()

content = content.replace(
    "ConstValue::Int(n) => builder.ins().iconst(types::I64, *n),",
    """ConstValue::Int(n) => {
                            let f = builder.ins().f64const(*n as f64);
                            builder.ins().bitcast(types::I64, MemFlags::new(), f)
                        }"""
)

with open("crates/backend-cranelift/src/aot.rs", "w") as f:
    f.write(content)
