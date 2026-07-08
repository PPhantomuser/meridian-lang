use meridian_vm::{Value, VM};
use std::fs;

pub fn register_all(vm: &mut VM) {
    vm.register_native_function(
        "read_file",
        Box::new(|args: &[Value]| {
            if args.len() != 1 {
                panic!("read_file expects exactly 1 argument");
            }
            if let Value::String(path) = &args[0] {
                match fs::read_to_string(path) {
                    Ok(content) => Value::String(content),
                    Err(_) => Value::Null, // Simple error handling for now
                }
            } else {
                panic!("read_file expects a String argument");
            }
        }),
    );

    // HashMap
    vm.register_native_function(
        "hashmap_new",
        Box::new(|args: &[Value]| {
            if !args.is_empty() {
                panic!("hashmap_new expects 0 arguments");
            }
            let map: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
            Value::NativeObject(std::sync::Arc::new(std::sync::Mutex::new(map)))
        }),
    );

    vm.register_native_function(
        "hashmap_insert",
        Box::new(|args: &[Value]| {
            if args.len() != 3 {
                panic!("hashmap_insert expects 3 arguments (map, key, value)");
            }
            if let Value::NativeObject(obj) = &args[0] {
                if let Value::String(key) = &args[1] {
                    let mut guard = obj.lock().unwrap();
                    if let Some(map) = guard.downcast_mut::<std::collections::HashMap<String, Value>>() {
                        map.insert(key.clone(), args[2].clone());
                        return Value::Null;
                    }
                }
            }
            panic!("Invalid arguments to hashmap_insert");
        }),
    );

    vm.register_native_function(
        "hashmap_get",
        Box::new(|args: &[Value]| {
            if args.len() != 2 {
                panic!("hashmap_get expects 2 arguments (map, key)");
            }
            if let Value::NativeObject(obj) = &args[0] {
                if let Value::String(key) = &args[1] {
                    let guard = obj.lock().unwrap();
                    if let Some(map) = guard.downcast_ref::<std::collections::HashMap<String, Value>>() {
                        if let Some(val) = map.get(key) {
                            return val.clone();
                        } else {
                            return Value::Null;
                        }
                    }
                }
            }
            panic!("Invalid arguments to hashmap_get");
        }),
    );

    // HashSet
    vm.register_native_function(
        "hashset_new",
        Box::new(|args: &[Value]| {
            if !args.is_empty() {
                panic!("hashset_new expects 0 arguments");
            }
            let set: std::collections::HashSet<String> = std::collections::HashSet::new();
            Value::NativeObject(std::sync::Arc::new(std::sync::Mutex::new(set)))
        }),
    );

    vm.register_native_function(
        "hashset_insert",
        Box::new(|args: &[Value]| {
            if args.len() != 2 {
                panic!("hashset_insert expects 2 arguments (set, value)");
            }
            if let Value::NativeObject(obj) = &args[0] {
                if let Value::String(val) = &args[1] {
                    let mut guard = obj.lock().unwrap();
                    if let Some(set) = guard.downcast_mut::<std::collections::HashSet<String>>() {
                        set.insert(val.clone());
                        return Value::Null;
                    }
                }
            }
            panic!("Invalid arguments to hashset_insert");
        }),
    );

    vm.register_native_function(
        "hashset_contains",
        Box::new(|args: &[Value]| {
            if args.len() != 2 {
                panic!("hashset_contains expects 2 arguments (set, value)");
            }
            if let Value::NativeObject(obj) = &args[0] {
                if let Value::String(val) = &args[1] {
                    let guard = obj.lock().unwrap();
                    if let Some(set) = guard.downcast_ref::<std::collections::HashSet<String>>() {
                        return Value::Bool(set.contains(val));
                    }
                }
            }
            panic!("Invalid arguments to hashset_contains");
        }),
    );

    // VecDeque
    vm.register_native_function(
        "vecdeque_new",
        Box::new(|args: &[Value]| {
            if !args.is_empty() {
                panic!("vecdeque_new expects 0 arguments");
            }
            let deque: std::collections::VecDeque<Value> = std::collections::VecDeque::new();
            Value::NativeObject(std::sync::Arc::new(std::sync::Mutex::new(deque)))
        }),
    );

    vm.register_native_function(
        "vecdeque_push_back",
        Box::new(|args: &[Value]| {
            if args.len() != 2 {
                panic!("vecdeque_push_back expects 2 arguments (deque, value)");
            }
            if let Value::NativeObject(obj) = &args[0] {
                let mut guard = obj.lock().unwrap();
                if let Some(deque) = guard.downcast_mut::<std::collections::VecDeque<Value>>() {
                    deque.push_back(args[1].clone());
                    return Value::Null;
                }
            }
            panic!("Invalid arguments to vecdeque_push_back");
        }),
    );

    vm.register_native_function(
        "vecdeque_pop_front",
        Box::new(|args: &[Value]| {
            if args.len() != 1 {
                panic!("vecdeque_pop_front expects 1 argument (deque)");
            }
            if let Value::NativeObject(obj) = &args[0] {
                let mut guard = obj.lock().unwrap();
                if let Some(deque) = guard.downcast_mut::<std::collections::VecDeque<Value>>() {
                    if let Some(val) = deque.pop_front() {
                        return val;
                    } else {
                        return Value::Null;
                    }
                }
            }
            panic!("Invalid arguments to vecdeque_pop_front");
        }),
    );

    // JSON
    vm.register_native_function(
        "json_parse",
        Box::new(|args: &[Value]| {
            if args.len() != 1 {
                panic!("json_parse expects 1 argument (json_string)");
            }
            if let Value::String(s) = &args[0] {
                let parsed: Result<serde_json::Value, _> = serde_json::from_str(s);
                match parsed {
                    Ok(val) => Value::NativeObject(std::sync::Arc::new(std::sync::Mutex::new(val))),
                    Err(_) => Value::Null,
                }
            } else {
                panic!("json_parse expects a string");
            }
        }),
    );

    vm.register_native_function(
        "json_stringify",
        Box::new(|args: &[Value]| {
            if args.len() != 1 {
                panic!("json_stringify expects 1 argument (json_object)");
            }
            if let Value::NativeObject(obj) = &args[0] {
                let guard = obj.lock().unwrap();
                if let Some(val) = guard.downcast_ref::<serde_json::Value>() {
                    let s = serde_json::to_string(val).unwrap_or_else(|_| "null".to_string());
                    return Value::String(s);
                }
            }
            panic!("json_stringify expects a json object");
        }),
    );

    // --- std::net ---
    vm.register_native_function(
        "tcp_bind",
        Box::new(|args: &[Value]| {
            if args.len() != 1 {
                panic!("tcp_bind expects 1 argument (address)");
            }
            if let Value::String(addr) = &args[0] {
                match std::net::TcpListener::bind(addr) {
                    Ok(listener) => {
                        Value::NativeObject(std::sync::Arc::new(std::sync::Mutex::new(listener)))
                    }
                    Err(e) => {
                        println!("tcp_bind error: {}", e);
                        Value::Null
                    }
                }
            } else {
                panic!("tcp_bind expects a String address");
            }
        }),
    );

    vm.register_native_function(
        "tcp_accept",
        Box::new(|args: &[Value]| {
            if args.len() != 1 {
                panic!("tcp_accept expects 1 argument (listener)");
            }
            if let Value::NativeObject(obj) = &args[0] {
                let mut guard = obj.lock().unwrap();
                if let Some(listener) = guard.downcast_mut::<std::net::TcpListener>() {
                    match listener.accept() {
                        Ok((stream, _addr)) => {
                            Value::NativeObject(std::sync::Arc::new(std::sync::Mutex::new(stream)))
                        }
                        Err(e) => {
                            println!("tcp_accept error: {}", e);
                            Value::Null
                        }
                    }
                } else {
                    panic!("tcp_accept expects a TcpListener");
                }
            } else {
                panic!("tcp_accept expects a TcpListener NativeObject");
            }
        }),
    );

    vm.register_native_function(
        "tcp_read",
        Box::new(|args: &[Value]| {
            use std::io::Read;
            if args.len() != 1 {
                panic!("tcp_read expects 1 argument (stream)");
            }
            if let Value::NativeObject(obj) = &args[0] {
                let mut guard = obj.lock().unwrap();
                if let Some(stream) = guard.downcast_mut::<std::net::TcpStream>() {
                    let mut buffer = [0; 1024];
                    match stream.read(&mut buffer) {
                        Ok(size) => {
                            let s = String::from_utf8_lossy(&buffer[..size]).into_owned();
                            Value::String(s)
                        }
                        Err(e) => {
                            println!("tcp_read error: {}", e);
                            Value::Null
                        }
                    }
                } else {
                    panic!("tcp_read expects a TcpStream");
                }
            } else {
                panic!("tcp_read expects a TcpStream NativeObject");
            }
        }),
    );

    vm.register_native_function(
        "tcp_write",
        Box::new(|args: &[Value]| {
            use std::io::Write;
            if args.len() != 2 {
                panic!("tcp_write expects 2 arguments (stream, data)");
            }
            if let Value::NativeObject(obj) = &args[0] {
                if let Value::String(data) = &args[1] {
                    let mut guard = obj.lock().unwrap();
                    if let Some(stream) = guard.downcast_mut::<std::net::TcpStream>() {
                        match stream.write_all(data.as_bytes()) {
                            Ok(_) => Value::Bool(true),
                            Err(e) => {
                                println!("tcp_write error: {}", e);
                                Value::Bool(false)
                            }
                        }
                    } else {
                        panic!("tcp_write expects a TcpStream");
                    }
                } else {
                    panic!("tcp_write data must be String");
                }
            } else {
                panic!("tcp_write expects a TcpStream NativeObject");
            }
        }),
    );

    // --- std::fs (extended) ---
    vm.register_native_function(
        "file_write",
        Box::new(|args: &[Value]| {
            if args.len() != 2 {
                panic!("file_write expects 2 arguments (path, content)");
            }
            if let (Value::String(path), Value::String(content)) = (&args[0], &args[1]) {
                match std::fs::write(path, content) {
                    Ok(_) => Value::Bool(true),
                    Err(_) => Value::Bool(false),
                }
            } else {
                panic!("file_write arguments must be String");
            }
        }),
    );

    vm.register_native_function(
        "file_append",
        Box::new(|args: &[Value]| {
            use std::io::Write;
            if args.len() != 2 {
                panic!("file_append expects 2 arguments (path, content)");
            }
            if let (Value::String(path), Value::String(content)) = (&args[0], &args[1]) {
                match std::fs::OpenOptions::new().create(true).append(true).open(path) {
                    Ok(mut file) => {
                        match file.write_all(content.as_bytes()) {
                            Ok(_) => Value::Bool(true),
                            Err(_) => Value::Bool(false),
                        }
                    }
                    Err(_) => Value::Bool(false),
                }
            } else {
                panic!("file_append arguments must be String");
            }
        }),
    );

    vm.register_native_function(
        "file_delete",
        Box::new(|args: &[Value]| {
            if args.len() != 1 {
                panic!("file_delete expects 1 argument (path)");
            }
            if let Value::String(path) = &args[0] {
                match std::fs::remove_file(path) {
                    Ok(_) => Value::Bool(true),
                    Err(_) => Value::Bool(false),
                }
            } else {
                panic!("file_delete argument must be String");
            }
        }),
    );

    vm.register_native_function(
        "file_exists",
        Box::new(|args: &[Value]| {
            if args.len() != 1 {
                panic!("file_exists expects 1 argument (path)");
            }
            if let Value::String(path) = &args[0] {
                Value::Bool(std::path::Path::new(path).exists())
            } else {
                panic!("file_exists argument must be String");
            }
        }),
    );

    // --- std::process ---
    vm.register_native_function(
        "process_output",
        Box::new(|args: &[Value]| {
            if args.len() != 2 {
                panic!("process_output expects 2 arguments (command, vecdeque_args)");
            }
            if let Value::String(cmd) = &args[0] {
                let mut command = std::process::Command::new(cmd);
                if let Value::NativeObject(obj) = &args[1] {
                    let guard = obj.lock().unwrap();
                    if let Some(deque) = guard.downcast_ref::<std::collections::VecDeque<Value>>() {
                        for arg in deque.iter() {
                            if let Value::String(s) = arg {
                                command.arg(s);
                            }
                        }
                    } else {
                        panic!("process_output args must be a VecDeque");
                    }
                } else {
                    panic!("process_output args must be a VecDeque NativeObject");
                }
                match command.output() {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
                        Value::String(stdout)
                    }
                    Err(e) => {
                        println!("process_output error: {}", e);
                        Value::Null
                    }
                }
            } else {
                panic!("process_output command must be String");
            }
        }),
    );
    // --- FFI (Foreign Function Interface) ---
    vm.register_native_function(
        "dlopen",
        Box::new(|args: &[Value]| {
            if args.len() != 1 {
                panic!("dlopen expects 1 argument (path)");
            }
            if let Value::String(path) = &args[0] {
                unsafe {
                    match libloading::Library::new(path) {
                        Ok(lib) => Value::NativeObject(std::sync::Arc::new(std::sync::Mutex::new(lib))),
                        Err(e) => {
                            println!("dlopen error: {}", e);
                            Value::Null
                        }
                    }
                }
            } else {
                panic!("dlopen argument must be String");
            }
        }),
    );

    vm.register_native_function(
        "dlsym",
        Box::new(|args: &[Value]| {
            if args.len() != 3 {
                panic!("dlsym expects 3 arguments (library, symbol_name, signature)");
            }
            if let Value::NativeObject(obj) = &args[0] {
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
                                    return Value::NativeObject(std::sync::Arc::new(std::sync::Mutex::new(func)));
                                }
                                Err(e) => {
                                    println!("dlsym error: {}", e);
                                    return Value::Null;
                                }
                            }
                        }
                    } else {
                        panic!("dlsym expects a libloading::Library NativeObject");
                    }
                } else {
                    panic!("dlsym arguments must be String");
                }
            } else {
                panic!("dlsym expects a Library NativeObject");
            }
        }),
    );

    vm.register_native_function(
        "dlcall",
        Box::new(|args: &[Value]| {
            if args.len() != 2 {
                panic!("dlcall expects 2 arguments (function, vecdeque_args)");
            }
            if let Value::NativeObject(obj) = &args[0] {
                let guard = obj.lock().unwrap();
                if let Some(func) = guard.downcast_ref::<FFIFunction>() {
                    if let Value::NativeObject(args_obj) = &args[1] {
                        let args_guard = args_obj.lock().unwrap();
                        if let Some(deque) = args_guard.downcast_ref::<std::collections::VecDeque<Value>>() {
                            // Match signatures
                            match func.signature.as_str() {
                                "f64,f64->f64" => {
                                    if deque.len() != 2 { panic!("dlcall signature f64,f64->f64 requires 2 arguments"); }
                                    if let (Value::Number(a), Value::Number(b)) = (&deque[0], &deque[1]) {
                                        let c_fn: unsafe extern "C" fn(f64, f64) -> f64 = unsafe { std::mem::transmute(func.ptr) };
                                        let res = unsafe { c_fn(*a, *b) };
                                        return Value::Number(res);
                                    } else {
                                        panic!("dlcall arguments do not match signature");
                                    }
                                }
                                "string->string" => {
                                    if deque.len() != 1 { panic!("dlcall signature string->string requires 1 argument"); }
                                    if let Value::String(s) = &deque[0] {
                                        let c_str = std::ffi::CString::new(s.as_str()).unwrap();
                                        let c_fn: unsafe extern "C" fn(*const std::ffi::c_char) -> *const std::ffi::c_char = unsafe { std::mem::transmute(func.ptr) };
                                        let res_ptr = unsafe { c_fn(c_str.as_ptr()) };
                                        if res_ptr.is_null() {
                                            return Value::Null;
                                        }
                                        let res_c_str = unsafe { std::ffi::CStr::from_ptr(res_ptr) };
                                        return Value::String(res_c_str.to_string_lossy().into_owned());
                                    } else {
                                        panic!("dlcall arguments do not match signature");
                                    }
                                }
                                _ => panic!("dlcall unsupported signature: {}", func.signature),
                            }
                        } else {
                            panic!("dlcall args must be a VecDeque");
                        }
                    } else {
                        panic!("dlcall args must be a VecDeque NativeObject");
                    }
                } else {
                    panic!("dlcall expects a FFIFunction NativeObject");
                }
            } else {
                panic!("dlcall expects a NativeObject");
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
