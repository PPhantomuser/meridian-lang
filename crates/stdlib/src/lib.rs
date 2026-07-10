use meridian_vm::{Value, VM};
use std::fs;

pub fn register_all(vm: &mut VM) {
    vm.register_native_function(
        "read_file",
        Box::new(|args: &[Value]| {
            if args.len() != 1 {
                return Value::Error("read_file expects exactly 1 argument".to_string());
            }
            if let Value::String(path) = &args[0] {
                match fs::read_to_string(path) {
                    Ok(content) => Value::Enum("Result".into(), "Ok".into(), Some(Box::new(Value::String(content)))),
                    Err(e) => Value::Enum("Result".into(), "Err".into(), Some(Box::new(Value::String(e.to_string())))),
                }
            } else {
                return Value::Error("read_file expects a String argument".to_string());
            }
        }),
    );

    // assert
    vm.register_native_function(
        "assert",
        Box::new(|args: &[Value]| {
            if args.is_empty() {
                return Value::Error("assert expects 1 argument".to_string());
            }
            if let Value::Bool(b) = args[0] {
                if b {
                    Value::Null
                } else {
                    Value::Error("Assertion failed".to_string())
                }
            } else {
                Value::Error("assert expects a boolean argument".to_string())
            }
        }),
    );

    // hashmap_new
    vm.register_native_function(
        "hashmap_new",
        Box::new(|args: &[Value]| {
            if !args.is_empty() {
                return Value::Error("hashmap_new expects 0 arguments".to_string());
            }
            let map: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
            Value::NativeObject("HashMap".to_string(), std::sync::Arc::new(std::sync::Mutex::new(map)))
        }),
    );

    vm.register_native_function(
        "hashmap_insert",
        Box::new(|args: &[Value]| {
            if args.len() != 3 {
                return Value::Error("hashmap_insert expects 3 arguments (map, key, value)".to_string());
            }
            if let Value::NativeObject(_, obj) = &args[0] {
                if let Value::String(key) = &args[1] {
                    let mut guard = obj.lock().unwrap();
                    if let Some(map) = guard.downcast_mut::<std::collections::HashMap<String, Value>>() {
                        map.insert(key.clone(), args[2].clone());
                        return Value::Null;
                    }
                }
            }
            return Value::Error("Invalid arguments to hashmap_insert".to_string());
        }),
    );

    vm.register_native_function(
        "hashmap_get",
        Box::new(|args: &[Value]| {
            if args.len() != 2 {
                return Value::Error("hashmap_get expects 2 arguments (map, key)".to_string());
            }
            if let Value::NativeObject(_, obj) = &args[0] {
                if let Value::String(key) = &args[1] {
                    let guard = obj.lock().unwrap();
                    if let Some(map) = guard.downcast_ref::<std::collections::HashMap<String, Value>>() {
                        if let Some(val) = map.get(key) {
                            return Value::Enum("Option".into(), "Some".into(), Some(Box::new(val.clone())));
                        } else {
                            return Value::Enum("Option".into(), "None".into(), None);
                        }
                    }
                }
            }
            return Value::Error("Invalid arguments to hashmap_get".to_string());
        }),
    );

    // HashSet
    vm.register_native_function(
        "hashset_new",
        Box::new(|args: &[Value]| {
            if !args.is_empty() {
                return Value::Error("hashset_new expects 0 arguments".to_string());
            }
            let set: std::collections::HashSet<String> = std::collections::HashSet::new();
            Value::NativeObject("HashSet".to_string(), std::sync::Arc::new(std::sync::Mutex::new(set)))
        }),
    );

    vm.register_native_function(
        "hashset_insert",
        Box::new(|args: &[Value]| {
            if args.len() != 2 {
                return Value::Error("hashset_insert expects 2 arguments (set, value)".to_string());
            }
            if let Value::NativeObject(_, obj) = &args[0] {
                if let Value::String(val) = &args[1] {
                    let mut guard = obj.lock().unwrap();
                    if let Some(set) = guard.downcast_mut::<std::collections::HashSet<String>>() {
                        set.insert(val.clone());
                        return Value::Null;
                    }
                }
            }
            return Value::Error("Invalid arguments to hashset_insert".to_string());
        }),
    );

    vm.register_native_function(
        "hashset_contains",
        Box::new(|args: &[Value]| {
            if args.len() != 2 {
                return Value::Error("hashset_contains expects 2 arguments (set, value)".to_string());
            }
            if let Value::NativeObject(_, obj) = &args[0] {
                if let Value::String(val) = &args[1] {
                    let guard = obj.lock().unwrap();
                    if let Some(set) = guard.downcast_ref::<std::collections::HashSet<String>>() {
                        return Value::Bool(set.contains(val));
                    }
                }
            }
            return Value::Error("Invalid arguments to hashset_contains".to_string());
        }),
    );

    // VecDeque
    vm.register_native_function(
        "vecdeque_new",
        Box::new(|args: &[Value]| {
            if !args.is_empty() {
                return Value::Error("vecdeque_new expects 0 arguments".to_string());
            }
            let deque: std::collections::VecDeque<Value> = std::collections::VecDeque::new();
            Value::NativeObject("VecDeque".to_string(), std::sync::Arc::new(std::sync::Mutex::new(deque)))
        }),
    );

    vm.register_native_function(
        "vecdeque_push_back",
        Box::new(|args: &[Value]| {
            if args.len() != 2 {
                return Value::Error("vecdeque_push_back expects 2 arguments (deque, value)".to_string());
            }
            if let Value::NativeObject(_, obj) = &args[0] {
                let mut guard = obj.lock().unwrap();
                if let Some(deque) = guard.downcast_mut::<std::collections::VecDeque<Value>>() {
                    deque.push_back(args[1].clone());
                    return Value::Null;
                }
            }
            return Value::Error("Invalid arguments to vecdeque_push_back".to_string());
        }),
    );

    vm.register_native_function(
        "vecdeque_pop_front",
        Box::new(|args: &[Value]| {
            if args.len() != 1 {
                return Value::Error("vecdeque_pop_front expects 1 argument (deque)".to_string());
            }
            if let Value::NativeObject(_, obj) = &args[0] {
                let mut guard = obj.lock().unwrap();
                if let Some(deque) = guard.downcast_mut::<std::collections::VecDeque<Value>>() {
                    if let Some(val) = deque.pop_front() {
                        return Value::Enum("Option".into(), "Some".into(), Some(Box::new(val)));
                    } else {
                        return Value::Enum("Option".into(), "None".into(), None);
                    }
                }
            }
            return Value::Error("Invalid arguments to vecdeque_pop_front".to_string());
        }),
    );

    // JSON
    vm.register_native_function(
        "json_parse",
        Box::new(|args: &[Value]| {
            if args.len() != 1 {
                return Value::Error("json_parse expects 1 argument (json_string)".to_string());
            }
            if let Value::String(s) = &args[0] {
                let parsed: Result<serde_json::Value, _> = serde_json::from_str(s);
                match parsed {
                    Ok(val) => Value::NativeObject("Unknown".to_string(), std::sync::Arc::new(std::sync::Mutex::new(val))),
                    Err(_) => Value::Null,
                }
            } else {
                return Value::Error("json_parse expects a string".to_string());
            }
        }),
    );

    vm.register_native_function(
        "json_stringify",
        Box::new(|args: &[Value]| {
            if args.len() != 1 {
                return Value::Error("json_stringify expects 1 argument (json_object)".to_string());
            }
            if let Value::NativeObject(_, obj) = &args[0] {
                let guard = obj.lock().unwrap();
                if let Some(val) = guard.downcast_ref::<serde_json::Value>() {
                    let s = serde_json::to_string(val).unwrap_or_else(|_| "null".to_string());
                    return Value::String(s);
                }
            }
            return Value::Error("json_stringify expects a json object".to_string());
        }),
    );

    // --- std::net ---
    vm.register_native_function(
        "tcp_bind",
        Box::new(|args: &[Value]| {
            if args.len() != 1 {
                return Value::Error("tcp_bind expects 1 argument (address)".to_string());
            }
            if let Value::String(addr) = &args[0] {
                match std::net::TcpListener::bind(addr) {
                    Ok(listener) => {
                        Value::Enum("Result".into(), "Ok".into(), Some(Box::new(Value::NativeObject("TcpListener".to_string(), std::sync::Arc::new(std::sync::Mutex::new(listener))))))
                    }
                    Err(e) => {
                        Value::Enum("Result".into(), "Err".into(), Some(Box::new(Value::String(e.to_string()))))
                    }
                }
            } else {
                return Value::Error("tcp_bind expects a String address".to_string());
            }
        }),
    );

    vm.register_native_function(
        "tcp_accept",
        Box::new(|args: &[Value]| {
            if args.len() != 1 {
                return Value::Error("tcp_accept expects 1 argument (listener)".to_string());
            }
            if let Value::NativeObject(_, obj) = &args[0] {
                let mut guard = obj.lock().unwrap();
                if let Some(listener) = guard.downcast_mut::<std::net::TcpListener>() {
                    match listener.accept() {
                        Ok((stream, _addr)) => {
                            Value::Enum("Result".into(), "Ok".into(), Some(Box::new(Value::NativeObject("TcpStream".to_string(), std::sync::Arc::new(std::sync::Mutex::new(stream))))))
                        }
                        Err(e) => {
                            Value::Enum("Result".into(), "Err".into(), Some(Box::new(Value::String(e.to_string()))))
                        }
                    }
                } else {
                    return Value::Error("tcp_accept expects a TcpListener".to_string());
                }
            } else {
                return Value::Error("tcp_accept expects a TcpListener NativeObject".to_string());
            }
        }),
    );

    vm.register_native_function(
        "tcp_read",
        Box::new(|args: &[Value]| {
            use std::io::Read;
            if args.len() != 1 {
                return Value::Error("tcp_read expects 1 argument (stream)".to_string());
            }
            if let Value::NativeObject(_, obj) = &args[0] {
                let mut guard = obj.lock().unwrap();
                if let Some(stream) = guard.downcast_mut::<std::net::TcpStream>() {
                    let mut buffer = [0; 1024];
                    match stream.read(&mut buffer) {
                        Ok(size) => {
                            let s = String::from_utf8_lossy(&buffer[..size]).into_owned();
                            Value::Enum("Result".into(), "Ok".into(), Some(Box::new(Value::String(s))))
                        }
                        Err(e) => {
                            Value::Enum("Result".into(), "Err".into(), Some(Box::new(Value::String(e.to_string()))))
                        }
                    }
                } else {
                    return Value::Error("tcp_read expects a TcpStream".to_string());
                }
            } else {
                return Value::Error("tcp_read expects a TcpStream NativeObject".to_string());
            }
        }),
    );

    vm.register_native_function(
        "tcp_write",
        Box::new(|args: &[Value]| {
            use std::io::Write;
            if args.len() != 2 {
                return Value::Error("tcp_write expects 2 arguments (stream, data)".to_string());
            }
            if let Value::NativeObject(_, obj) = &args[0] {
                if let Value::String(data) = &args[1] {
                    let mut guard = obj.lock().unwrap();
                    if let Some(stream) = guard.downcast_mut::<std::net::TcpStream>() {
                        match stream.write_all(data.as_bytes()) {
                            Ok(_) => Value::Enum("Result".into(), "Ok".into(), Some(Box::new(Value::Bool(true)))),
                            Err(e) => {
                                Value::Enum("Result".into(), "Err".into(), Some(Box::new(Value::String(e.to_string()))))
                            }
                        }
                    } else {
                        return Value::Error("tcp_write expects a TcpStream".to_string());
                    }
                } else {
                    return Value::Error("tcp_write data must be String".to_string());
                }
            } else {
                return Value::Error("tcp_write expects a TcpStream NativeObject".to_string());
            }
        }),
    );

    // --- std::fs (extended) ---
    vm.register_native_function(
        "file_write",
        Box::new(|args: &[Value]| {
            if args.len() != 2 {
                return Value::Error("file_write expects 2 arguments (path, content)".to_string());
            }
            if let (Value::String(path), Value::String(content)) = (&args[0], &args[1]) {
                match std::fs::write(path, content) {
                    Ok(_) => Value::Enum("Result".into(), "Ok".into(), Some(Box::new(Value::Bool(true)))),
                    Err(e) => Value::Enum("Result".into(), "Err".into(), Some(Box::new(Value::String(e.to_string())))),
                }
            } else {
                return Value::Error("file_write arguments must be String".to_string());
            }
        }),
    );

    vm.register_native_function(
        "file_append",
        Box::new(|args: &[Value]| {
            use std::io::Write;
            if args.len() != 2 {
                return Value::Error("file_append expects 2 arguments (path, content)".to_string());
            }
            if let (Value::String(path), Value::String(content)) = (&args[0], &args[1]) {
                match std::fs::OpenOptions::new().create(true).append(true).open(path) {
                    Ok(mut file) => {
                        match file.write_all(content.as_bytes()) {
                            Ok(_) => Value::Enum("Result".into(), "Ok".into(), Some(Box::new(Value::Bool(true)))),
                            Err(e) => Value::Enum("Result".into(), "Err".into(), Some(Box::new(Value::String(e.to_string())))),
                        }
                    }
                    Err(e) => Value::Enum("Result".into(), "Err".into(), Some(Box::new(Value::String(e.to_string())))),
                }
            } else {
                return Value::Error("file_append arguments must be String".to_string());
            }
        }),
    );

    vm.register_native_function(
        "file_delete",
        Box::new(|args: &[Value]| {
            if args.len() != 1 {
                return Value::Error("file_delete expects 1 argument (path)".to_string());
            }
            if let Value::String(path) = &args[0] {
                match std::fs::remove_file(path) {
                    Ok(_) => Value::Enum("Result".into(), "Ok".into(), Some(Box::new(Value::Bool(true)))),
                    Err(e) => Value::Enum("Result".into(), "Err".into(), Some(Box::new(Value::String(e.to_string())))),
                }
            } else {
                return Value::Error("file_delete argument must be String".to_string());
            }
        }),
    );

    vm.register_native_function(
        "file_exists",
        Box::new(|args: &[Value]| {
            if args.len() != 1 {
                return Value::Error("file_exists expects 1 argument (path)".to_string());
            }
            if let Value::String(path) = &args[0] {
                Value::Bool(std::path::Path::new(path).exists())
            } else {
                return Value::Error("file_exists argument must be String".to_string());
            }
        }),
    );

    // --- std::process ---
    vm.register_native_function(
        "process_output",
        Box::new(|args: &[Value]| {
            if args.len() != 2 {
                return Value::Error("process_output expects 2 arguments (command, vecdeque_args)".to_string());
            }
            if let Value::String(cmd) = &args[0] {
                let mut command = std::process::Command::new(cmd);
                if let Value::NativeObject(_, obj) = &args[1] {
                    let guard = obj.lock().unwrap();
                    if let Some(deque) = guard.downcast_ref::<std::collections::VecDeque<Value>>() {
                        for arg in deque.iter() {
                            if let Value::String(s) = arg {
                                command.arg(s);
                            }
                        }
                    } else {
                        return Value::Error("process_output args must be a VecDeque".to_string());
                    }
                } else {
                    return Value::Error("process_output args must be a VecDeque NativeObject".to_string());
                }
                match command.output() {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
                        Value::Enum("Result".into(), "Ok".into(), Some(Box::new(Value::String(stdout))))
                    }
                    Err(e) => {
                        Value::Enum("Result".into(), "Err".into(), Some(Box::new(Value::String(e.to_string()))))
                    }
                }
            } else {
                return Value::Error("process_output command must be String".to_string());
            }
        }),
    );
    // --- FFI (Foreign Function Interface) ---
    vm.register_native_function(
        "dlopen",
        Box::new(|args: &[Value]| {
            if args.len() != 1 {
                return Value::Error("dlopen expects 1 argument (path)".to_string());
            }
            if let Value::String(path) = &args[0] {
                unsafe {
                    match libloading::Library::new(path) {
                        Ok(lib) => Value::Enum("Result".into(), "Ok".into(), Some(Box::new(Value::NativeObject("Library".to_string(), std::sync::Arc::new(std::sync::Mutex::new(lib)))))),
                        Err(e) => {
                            Value::Enum("Result".into(), "Err".into(), Some(Box::new(Value::String(e.to_string()))))
                        }
                    }
                }
            } else {
                return Value::Error("dlopen argument must be String".to_string());
            }
        }),
    );

    vm.register_native_function(
        "dlsym",
        Box::new(|args: &[Value]| {
            if args.len() != 3 {
                return Value::Error("dlsym expects 3 arguments (library, symbol_name, signature)".to_string());
            }
            if let Value::NativeObject(_, obj) = &args[0] {
                if let (Value::String(sym), Value::String(sig)) = (&args[1], &args[2]) {
                    let guard = obj.lock().unwrap();
                    if let Some(lib) = guard.downcast_ref::<libloading::Library>() {
                        unsafe {
                            let sym_c = std::ffi::CString::new(sym.as_bytes()).unwrap();
                            match lib.get::<libloading::Symbol<unsafe extern "C" fn()>>(&sym_c.as_bytes_with_nul()) {
                                Ok(symbol) => {
                                    let ptr = symbol.into_raw().into_raw();
                                    let func = FFIFunction {
                                        ptr,
                                        signature: sig.clone(),
                                    };
                                    return Value::Enum("Option".into(), "Some".into(), Some(Box::new(Value::NativeObject("Function".to_string(), std::sync::Arc::new(std::sync::Mutex::new(func))))));
                                }
                                Err(_e) => {
                                    return Value::Enum("Option".into(), "None".into(), None);
                                }
                            }
                        }
                    } else {
                        return Value::Error("dlsym expects a libloading::Library NativeObject".to_string());
                    }
                } else {
                    return Value::Error("dlsym arguments must be String".to_string());
                }
            } else {
                return Value::Error("dlsym expects a Library NativeObject".to_string());
            }
        }),
    );

    vm.register_native_function(
        "dlcall",
        Box::new(|args: &[Value]| {
            if args.len() != 2 {
                return Value::Error("dlcall expects 2 arguments (function, vecdeque_args)".to_string());
            }
            if let Value::NativeObject(_, obj) = &args[0] {
                let guard = obj.lock().unwrap();
                if let Some(func) = guard.downcast_ref::<FFIFunction>() {
                    if let Value::NativeObject(_, args_obj) = &args[1] {
                        let args_guard = args_obj.lock().unwrap();
                        if let Some(deque) = args_guard.downcast_ref::<std::collections::VecDeque<Value>>() {
                            // Match signatures
                            match func.signature.as_str() {
                                "f64,f64->f64" => {
                                    if deque.len() != 2 { return Value::Error("dlcall signature f64,f64->f64 requires 2 arguments".to_string()); }
                                    if let (Value::Number(a), Value::Number(b)) = (&deque[0], &deque[1]) {
                                        let c_fn: unsafe extern "C" fn(f64, f64) -> f64 = unsafe { std::mem::transmute(func.ptr) };
                                        let res = unsafe { c_fn(*a, *b) };
                                        return Value::Enum("Result".into(), "Ok".into(), Some(Box::new(Value::Number(res))));
                                    } else {
                                        return Value::Error("dlcall arguments do not match signature".to_string());
                                    }
                                }
                                "string->string" => {
                                    if deque.len() != 1 { return Value::Error("dlcall signature string->string requires 1 argument".to_string()); }
                                    if let Value::String(s) = &deque[0] {
                                        let c_str = std::ffi::CString::new(s.as_str()).unwrap();
                                        let c_fn: unsafe extern "C" fn(*const std::ffi::c_char) -> *const std::ffi::c_char = unsafe { std::mem::transmute(func.ptr) };
                                        let res_ptr = unsafe { c_fn(c_str.as_ptr()) };
                                        if res_ptr.is_null() {
                                            return Value::Enum("Result".into(), "Err".into(), Some(Box::new(Value::String("Null pointer".to_string()))));
                                        }
                                        let res_c_str = unsafe { std::ffi::CStr::from_ptr(res_ptr) };
                                        return Value::Enum("Result".into(), "Ok".into(), Some(Box::new(Value::String(res_c_str.to_string_lossy().into_owned()))));
                                    } else {
                                        return Value::Error("dlcall arguments do not match signature".to_string());
                                    }
                                }
                                _ => Value::Error(format!("dlcall unsupported signature: {}", func.signature)),
                            }
                        } else {
                            return Value::Error("dlcall args must be a VecDeque".to_string());
                        }
                    } else {
                        return Value::Error("dlcall args must be a VecDeque NativeObject".to_string());
                    }
                } else {
                    return Value::Error("dlcall expects a FFIFunction NativeObject".to_string());
                }
            } else {
                return Value::Error("dlcall expects a NativeObject".to_string());
            }
        }),
    );
}

pub struct FFIFunction {
    pub ptr: *mut std::ffi::c_void,
    pub signature: String,
}

unsafe impl Send for FFIFunction {}
unsafe impl Sync for FFIFunction {}
