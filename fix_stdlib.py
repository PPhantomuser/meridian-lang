import re

with open('crates/stdlib/src/lib.rs', 'r') as f:
    code = f.read()

# Replace Value::NativeObject(obj) extraction
# We don't actually need to enforce tag checking in the stdlib because downcast_mut will fail safely anyway!
# We just need to change the extraction syntax to `Value::NativeObject(_, obj)` so it compiles!
code = code.replace("Value::NativeObject(obj)", "Value::NativeObject(_, obj)")
code = code.replace("Value::NativeObject(args_obj)", "Value::NativeObject(_, args_obj)")

# Replace instantiation Value::NativeObject(std::sync::Arc::new(std::sync::Mutex::new(...)))
# This is a bit trickier, let's just do it manually for the ones we know
replacements = {
    "hashmap_new": ('Value::NativeObject(std::sync::Arc::new(std::sync::Mutex::new(map)))', 'Value::NativeObject("HashMap".to_string(), std::sync::Arc::new(std::sync::Mutex::new(map)))'),
    "hashset_new": ('Value::NativeObject(std::sync::Arc::new(std::sync::Mutex::new(set)))', 'Value::NativeObject("HashSet".to_string(), std::sync::Arc::new(std::sync::Mutex::new(set)))'),
    "vecdeque_new": ('Value::NativeObject(std::sync::Arc::new(std::sync::Mutex::new(deque)))', 'Value::NativeObject("VecDeque".to_string(), std::sync::Arc::new(std::sync::Mutex::new(deque)))'),
    "tcp_bind": ('Value::NativeObject(std::sync::Arc::new(std::sync::Mutex::new(listener)))', 'Value::NativeObject("TcpListener".to_string(), std::sync::Arc::new(std::sync::Mutex::new(listener)))'),
    "tcp_accept": ('Value::NativeObject(std::sync::Arc::new(std::sync::Mutex::new(stream)))', 'Value::NativeObject("TcpStream".to_string(), std::sync::Arc::new(std::sync::Mutex::new(stream)))'),
    "dlopen": ('Value::NativeObject(std::sync::Arc::new(std::sync::Mutex::new(lib)))', 'Value::NativeObject("Library".to_string(), std::sync::Arc::new(std::sync::Mutex::new(lib)))'),
    "dlsym": ('Value::NativeObject(std::sync::Arc::new(std::sync::Mutex::new(func)))', 'Value::NativeObject("Function".to_string(), std::sync::Arc::new(std::sync::Mutex::new(func)))'),
}

for r in replacements.values():
    code = code.replace(r[0], r[1])

# Also there's one in tcp_read for the process_output... wait process_output returns what?
# "process_output" returns native object? No, it returns a string in semantic. Let's see process_output in stdlib
# wait, there's `Ok(val) => Value::NativeObject(std::sync::Arc::new(std::sync::Mutex::new(val)))` maybe for tcp_accept?
# Let's just regex it!
code = re.sub(r'Value::NativeObject\((std::sync::Arc::new\(std::sync::Mutex::new\(val\)\))\)', r'Value::NativeObject("Unknown".to_string(), \1)', code)

with open('crates/stdlib/src/lib.rs', 'w') as f:
    f.write(code)

print("Done")
