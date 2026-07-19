use meridian_ir::{Chunk, ConstValue, Opcode, ProgramIR, Register};
use std::collections::{HashMap, VecDeque, HashSet};

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Int(i64),
    String(String),
    Bool(bool),
    Null,
    NativeFunction(usize),
    Reference(usize), // Index into the current task's registers
    Future(usize),    // TaskId
    NativeObject(String, std::sync::Arc<std::sync::Mutex<dyn std::any::Any + Send + Sync>>),
    Struct(String, std::sync::Arc<std::sync::Mutex<Vec<Value>>>),
    Array(std::sync::Arc<std::sync::Mutex<Vec<Value>>>),
    Enum(usize, Vec<Value>),
    Error(String),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Int(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
            Value::NativeFunction(_) => write!(f, "<native fn>"),
            Value::Reference(ptr) => write!(f, "<reference to {}>", ptr),
            Value::Future(id) => write!(f, "<Future task_id={}>", id),
            Value::NativeObject(tag, _) => write!(f, "<NativeObject: {}>", tag),
            Value::Struct(name, fields_arc) => {
                let fields = fields_arc.lock().unwrap();
                write!(f, "{} {{ ", name)?;
                let mut first = true;
                for v in fields.iter() {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                    first = false;
                }
                write!(f, " }}")
            }
            Value::Array(items_arc) => {
                let items = items_arc.lock().unwrap();
                write!(f, "[")?;
                let mut first = true;
                for item in items.iter() {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                    first = false;
                }
                write!(f, "]")
            }
            Value::Enum(variant_idx, values) => {
                write!(f, "Enum(Variant {}) {{ ", variant_idx)?;
                let mut first = true;
                for v in values {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                    first = false;
                }
                write!(f, " }}")
            }
            Value::Error(msg) => write!(f, "<Error: {}>", msg),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::NativeFunction(a), Value::NativeFunction(b)) => a == b,
            (Value::Reference(a), Value::Reference(b)) => a == b,
            (Value::Future(a), Value::Future(b)) => a == b,
            (Value::NativeObject(tag_a, a), Value::NativeObject(tag_b, b)) => tag_a == tag_b && std::sync::Arc::ptr_eq(a, b),
            (Value::Struct(_, a), Value::Struct(_, b)) => std::sync::Arc::ptr_eq(a, b),
            (Value::Array(a), Value::Array(b)) => std::sync::Arc::ptr_eq(a, b),
            (Value::Enum(idx1, val1), Value::Enum(idx2, val2)) => idx1 == idx2 && val1 == val2,
            (Value::Error(a), Value::Error(b)) => a == b,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub message: String,
    pub stack_trace: Vec<(String, usize)>,
}

#[derive(Clone)]
struct CallFrame {
    function_name: String,
    chunk: Chunk,
    ip: usize,
    base_register: usize,
    dest_register: Register,
}

#[derive(Clone)]
struct Task {
    id: usize,
    frames: Vec<CallFrame>,
    registers: Vec<Value>,
    result: Option<Value>,
    waiting_on: Option<(usize, usize)>,
}

pub struct VM {
    functions: HashMap<String, (Chunk, bool, usize)>,
    extern_functions: HashMap<String, usize>,
    #[allow(clippy::type_complexity)]
    native_functions: Vec<Box<dyn Fn(&[Value]) -> Value>>,
    global_env: HashMap<String, Value>,
    
    tasks: HashMap<usize, Task>,
    ready_queue: VecDeque<usize>,
    next_task_id: usize,
    
    pub coverage: HashMap<String, HashSet<usize>>,
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

impl VM {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            extern_functions: HashMap::new(),
            native_functions: Vec::new(),
            global_env: HashMap::new(),
            tasks: HashMap::new(),
            ready_queue: VecDeque::new(),
            next_task_id: 1,
            coverage: HashMap::new(),
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn register_native_function(&mut self, name: &str, func: Box<dyn Fn(&[Value]) -> Value>) {
        let idx = self.native_functions.len();
        self.native_functions.push(func);
        self.global_env.insert(name.to_string(), Value::NativeFunction(idx));
    }

    pub fn run(&mut self, program_ir: &ProgramIR) -> Result<(), RuntimeError> {
        self.functions = program_ir.functions.clone();
        self.extern_functions = program_ir.extern_functions.clone();
        
        let main_task = Task {
            id: 0,
            frames: vec![CallFrame {
                function_name: "main".to_string(),
                chunk: program_ir.main_chunk.clone(),
                ip: 0,
                base_register: 0,
                dest_register: 0,
            }],
            registers: vec![Value::Null; 10000],
            result: None,
            waiting_on: None,
        };
        
        self.tasks.insert(0, main_task);
        self.ready_queue.push_back(0);

        self.execute()
    }

    fn spawn_task(&mut self, function_name: String, chunk: Chunk, args: &[Value]) -> usize {
        let id = self.next_task_id;
        self.next_task_id += 1;
        
        let mut registers = vec![Value::Null; 10000];
        for (i, arg) in args.iter().enumerate() {
            registers[i] = arg.clone();
        }
        
        let task = Task {
            id,
            frames: vec![CallFrame {
                function_name,
                chunk,
                ip: 0,
                base_register: 0,
                dest_register: 0,
            }],
            registers,
            result: None,
            waiting_on: None,
        };
        
        self.tasks.insert(id, task);
        self.ready_queue.push_back(id);
        id
    }

    fn generate_stack_trace(&self, task: &Task, frame: &CallFrame) -> Vec<(String, usize)> {
        let mut trace = Vec::new();
        trace.push((frame.function_name.clone(), frame.ip));
        for f in task.frames.iter().rev() {
            trace.push((f.function_name.clone(), f.ip));
        }
        trace
    }

    fn execute(&mut self) -> Result<(), RuntimeError> {
        while let Some(task_id) = self.ready_queue.pop_front() {
            let mut task = self.tasks.remove(&task_id).unwrap();
            
            if let Some((wait_id, dest_reg)) = task.waiting_on {
                let wait_task = self.tasks.get(&wait_id).unwrap();
                if let Some(res) = &wait_task.result {
                    task.registers[dest_reg] = res.clone();
                    task.waiting_on = None;
                } else {
                    self.tasks.insert(task_id, task);
                    continue; 
                }
            }

            let mut yielded = false;

            while let Some(mut frame) = task.frames.pop() {
                let mut try_return_val = None;
                while frame.ip < frame.chunk.instructions.len() {
                    
                    // Code coverage tracking
                    self.coverage
                        .entry(frame.function_name.clone())
                        .or_default()
                        .insert(frame.ip);

                    let instruction = frame.chunk.instructions[frame.ip].clone();
                    frame.ip += 1;

                    let base = frame.base_register;

                    match instruction {
                        Opcode::LoadConst(dest, const_idx) => {
                            let c = &frame.chunk.constants[const_idx];
                            let val = match c {
                                ConstValue::Number(n) => Value::Number(*n),
                                ConstValue::Int(n) => Value::Int(*n),
                                ConstValue::String(s) => Value::String(s.clone()),
                                ConstValue::Bool(b) => Value::Bool(*b),
                            };
                            task.registers[base + dest] = val;
                        }

                        Opcode::Move(dest, src) => {
                            task.registers[base + dest] = task.registers[base + src].clone();
                        }
                        Opcode::Neg(dest, src, _) => {
                            if let Value::Number(v) = &task.registers[base + src] {
                                task.registers[base + dest] = Value::Number(-v);
                            } else if let Value::Int(v) = &task.registers[base + src] {
                                task.registers[base + dest] = Value::Int(-v);
                            } else {
                                return Err(RuntimeError {
                                    message: "Invalid type for negation".to_string(),
                                    stack_trace: self.generate_stack_trace(&task, &frame),
                                });
                            }
                        }
                        Opcode::Add(dest, left, right, _) => {
                            if let (Value::Number(l), Value::Number(r)) = (&task.registers[base + left], &task.registers[base + right]) {
                                task.registers[base + dest] = Value::Number(l + r);
                            } else if let (Value::Int(l), Value::Int(r)) = (&task.registers[base + left], &task.registers[base + right]) {
                                task.registers[base + dest] = Value::Int(l + r);
                            } else if let (Value::String(l), Value::String(r)) = (&task.registers[base + left], &task.registers[base + right]) {
                                task.registers[base + dest] = Value::String(format!("{}{}", l, r));
                            } else {
                                return Err(RuntimeError {
                                    message: "Invalid types for addition".to_string(),
                                    stack_trace: self.generate_stack_trace(&task, &frame),
                                });
                            }
                        }
                        Opcode::Sub(dest, left, right, _) => {
                            if let (Value::Number(l), Value::Number(r)) = (&task.registers[base + left], &task.registers[base + right]) {
                                task.registers[base + dest] = Value::Number(l - r);
                            } else if let (Value::Int(l), Value::Int(r)) = (&task.registers[base + left], &task.registers[base + right]) {
                                task.registers[base + dest] = Value::Int(l - r);
                            } else {
                                return Err(RuntimeError {
                                    message: "Invalid types for subtraction".to_string(),
                                    stack_trace: self.generate_stack_trace(&task, &frame),
                                });
                            }
                        }
                        Opcode::Mul(dest, left, right, _) => {
                            if let (Value::Number(l), Value::Number(r)) = (&task.registers[base + left], &task.registers[base + right]) {
                                task.registers[base + dest] = Value::Number(l * r);
                            } else if let (Value::Int(l), Value::Int(r)) = (&task.registers[base + left], &task.registers[base + right]) {
                                task.registers[base + dest] = Value::Int(l * r);
                            } else {
                                return Err(RuntimeError {
                                    message: "Invalid types for multiplication".to_string(),
                                    stack_trace: self.generate_stack_trace(&task, &frame),
                                });
                            }
                        }
                        Opcode::Div(dest, left, right, _) => {
                            if let (Value::Number(l), Value::Number(r)) = (&task.registers[base + left], &task.registers[base + right]) {
                                if *r == 0.0 {
                                    return Err(RuntimeError {
                                        message: "Division by zero".to_string(),
                                        stack_trace: self.generate_stack_trace(&task, &frame),
                                    });
                                }
                                task.registers[base + dest] = Value::Number(l / r);
                            } else if let (Value::Int(l), Value::Int(r)) = (&task.registers[base + left], &task.registers[base + right]) {
                                if *r == 0 {
                                    return Err(RuntimeError {
                                        message: "Division by zero".to_string(),
                                        stack_trace: self.generate_stack_trace(&task, &frame),
                                    });
                                }
                                task.registers[base + dest] = Value::Int(l / r);
                            } else {
                                return Err(RuntimeError {
                                    message: "Invalid types for division".to_string(),
                                    stack_trace: self.generate_stack_trace(&task, &frame),
                                });
                            }
                        }
                        Opcode::Eq(dest, left, right, _) => {
                            let eq = task.registers[base + left] == task.registers[base + right];
                            task.registers[base + dest] = Value::Bool(eq);
                        }
                        Opcode::Ne(dest, left, right, _) => {
                            let eq = task.registers[base + left] == task.registers[base + right];
                            task.registers[base + dest] = Value::Bool(!eq);
                        }
                        Opcode::Lt(dest, left, right, _) => {
                            if let (Value::Number(l), Value::Number(r)) = (&task.registers[base + left], &task.registers[base + right]) {
                                task.registers[base + dest] = Value::Bool(l < r);
                            } else if let (Value::Int(l), Value::Int(r)) = (&task.registers[base + left], &task.registers[base + right]) {
                                task.registers[base + dest] = Value::Bool(l < r);
                            } else {
                                task.registers[base + dest] = Value::Bool(false);
                            }
                        }
                        Opcode::Le(dest, left, right, _) => {
                            if let (Value::Number(l), Value::Number(r)) = (&task.registers[base + left], &task.registers[base + right]) {
                                task.registers[base + dest] = Value::Bool(l <= r);
                            } else if let (Value::Int(l), Value::Int(r)) = (&task.registers[base + left], &task.registers[base + right]) {
                                task.registers[base + dest] = Value::Bool(l <= r);
                            } else {
                                task.registers[base + dest] = Value::Bool(false);
                            }
                        }
                        Opcode::Gt(dest, left, right, _) => {
                            if let (Value::Number(l), Value::Number(r)) = (&task.registers[base + left], &task.registers[base + right]) {
                                task.registers[base + dest] = Value::Bool(l > r);
                            } else if let (Value::Int(l), Value::Int(r)) = (&task.registers[base + left], &task.registers[base + right]) {
                                task.registers[base + dest] = Value::Bool(l > r);
                            } else {
                                task.registers[base + dest] = Value::Bool(false);
                            }
                        }
                        Opcode::Ge(dest, left, right, _) => {
                            if let (Value::Number(l), Value::Number(r)) = (&task.registers[base + left], &task.registers[base + right]) {
                                task.registers[base + dest] = Value::Bool(l >= r);
                            } else if let (Value::Int(l), Value::Int(r)) = (&task.registers[base + left], &task.registers[base + right]) {
                                task.registers[base + dest] = Value::Bool(l >= r);
                            } else {
                                task.registers[base + dest] = Value::Bool(false);
                            }
                        }
                        Opcode::JumpIfFalse(cond, offset) => {
                            if let Value::Bool(b) = &task.registers[base + cond] {
                                if !b {
                                    frame.ip = offset;
                                }
                            }
                        }
                        Opcode::Jump(offset) => {
                            frame.ip = offset;
                        }
                        Opcode::Print(src, _) => {
                            println!("{}", &task.registers[base + src]);
                        }
                        Opcode::Call(dest, ref name, arg_start, arg_count) => {
                            if let Some(Value::NativeFunction(func_id)) = self.global_env.get(name) {
                                let mut args = Vec::new();
                                for j in 0..arg_count {
                                    args.push(task.registers[base + arg_start + j].clone());
                                }
                                let res = (self.native_functions[*func_id])(&args);
                                if let Value::Error(msg) = &res {
                                    return Err(RuntimeError {
                                        message: msg.clone(),
                                        stack_trace: self.generate_stack_trace(&task, &frame),
                                    });
                                }
                                task.registers[base + dest] = res;
                            } else if let Some((func_chunk, is_async, _)) = self.functions.get(name) {
                                if *is_async {
                                    let new_base = base + arg_start;
                                    let args = task.registers[new_base..new_base + arg_count].to_vec();
                                    let new_task_id = self.spawn_task(name.clone(), func_chunk.clone(), &args);
                                    task.registers[base + dest] = Value::Future(new_task_id);
                                } else {
                                    let new_base = base + 256; // Simple fixed frame size
                                    if new_base + 256 >= task.registers.len() {
                                        task.registers.resize(new_base + 1024, Value::Null);
                                    }
                                    for j in 0..arg_count {
                                        task.registers[new_base + j] = task.registers[base + arg_start + j].clone();
                                    }
                                    task.frames.push(frame);
                                    
                                    frame = CallFrame {
                                        function_name: name.clone(),
                                        chunk: func_chunk.clone(),
                                        ip: 0,
                                        base_register: new_base,
                                        dest_register: base + dest,
                                    };
                                }
                            } else if self.extern_functions.contains_key(name) {
                                return Err(RuntimeError {
                                    message: format!("Cannot execute external C function '{}' in interpreter mode.", name),
                                    stack_trace: self.generate_stack_trace(&task, &frame),
                                });
                            } else {
                                return Err(RuntimeError {
                                    message: format!("Function {} not found", name),
                                    stack_trace: self.generate_stack_trace(&task, &frame),
                                });
                            }
                        }
                        Opcode::AsyncCall(dest, func_name, arg_start, arg_count) => {
                            if let Some((func_chunk, _, _)) = self.functions.get(&func_name) {
                                let new_base = base + arg_start;
                                let args = task.registers[new_base..new_base + arg_count].to_vec();
                                let new_task_id = self.spawn_task(func_name.clone(), func_chunk.clone(), &args);
                                task.registers[base + dest] = Value::Future(new_task_id);
                            } else {
                                return Err(RuntimeError {
                                    message: format!("Async Block Function {} not found", func_name),
                                    stack_trace: self.generate_stack_trace(&task, &frame),
                                });
                            }
                        }
                        Opcode::Return(_) => {
                            break;
                        }
                        Opcode::Await(dest, src_future) => {
                            if let Value::Future(wait_id) = task.registers[base + src_future] {
                                task.waiting_on = Some((wait_id, base + dest));
                                task.frames.push(frame.clone()); // Save current frame
                                yielded = true;
                                break;
                            } else {
                                return Err(RuntimeError {
                                    message: "Awaiting a non-future".to_string(),
                                    stack_trace: self.generate_stack_trace(&task, &frame),
                                });
                            }
                        }
                        Opcode::Spawn(dest, src_future) => {
                            task.registers[base + dest] = task.registers[base + src_future].clone();
                        }
                        Opcode::Borrow(dest, src_reg) => {
                            let ptr_idx = base + src_reg;
                            task.registers[base + dest] = Value::Reference(ptr_idx);
                        }
                        Opcode::Dereference(dest, src_reg) => {
                            if let Value::Reference(ptr) = task.registers[base + src_reg] {
                                task.registers[base + dest] = task.registers[ptr].clone();
                            } else {
                                return Err(RuntimeError {
                                    message: "Dereferencing a non-reference value".to_string(),
                                    stack_trace: self.generate_stack_trace(&task, &frame),
                                });
                            }
                        }
                        Opcode::MakeStruct(dest, field_count, first_field_reg) => {
                            let mut map = Vec::new();
                            for i in 0..field_count {
                                map.push(task.registers[base + first_field_reg + i].clone());
                            }
                            task.registers[base + dest] = Value::Struct("".to_string(), std::sync::Arc::new(std::sync::Mutex::new(map)));
                        }
                        Opcode::FieldAccess(dest, obj_reg, field_idx) => {
                            let mut obj_val = task.registers[base + obj_reg].clone();
                            while let Value::Reference(ptr) = obj_val {
                                obj_val = task.registers[ptr].clone();
                            }
                            if let Value::Struct(_, map) = obj_val {
                                let map = map.lock().unwrap();
                                if field_idx < map.len() {
                                    task.registers[base + dest] = map[field_idx].clone();
                                } else {
                                    return Err(RuntimeError {
                                        message: format!("Struct missing field index {}", field_idx),
                                        stack_trace: self.generate_stack_trace(&task, &frame),
                                    });
                                }
                            } else {
                                return Err(RuntimeError {
                                    message: "FieldAccess on non-struct".to_string(),
                                    stack_trace: self.generate_stack_trace(&task, &frame),
                                });
                            }
                        }
                        Opcode::FieldAssign(obj_reg, field_idx, val_reg) => {
                            let val = task.registers[base + val_reg].clone();
                            let mut obj_ptr = base + obj_reg;
                            loop {
                                match &task.registers[obj_ptr] {
                                    Value::Reference(inner) => {
                                        obj_ptr = *inner;
                                    }
                                    _ => break,
                                }
                            }
                            
                            if let Value::Struct(_, map) = &task.registers[obj_ptr] {
                                let mut map = map.lock().unwrap();
                                if field_idx < map.len() {
                                    map[field_idx] = val;
                                }
                            } else {
                                return Err(RuntimeError {
                                    message: "FieldAssign on non-struct".to_string(),
                                    stack_trace: self.generate_stack_trace(&task, &frame),
                                });
                            }
                        }
                        Opcode::MakeArray(dest, first_elem_reg, count) => {
                            let mut elements = Vec::with_capacity(count);
                            for i in 0..count {
                                elements.push(task.registers[base + first_elem_reg + i].clone());
                            }
                            task.registers[base + dest] = Value::Array(std::sync::Arc::new(std::sync::Mutex::new(elements)));
                        }
                        Opcode::ArrayIndex(dest, obj_reg, index_reg) => {
                            let obj_val = task.registers[base + obj_reg].clone();
                            let index_val = task.registers[base + index_reg].clone();
                            
                            let idx = if let Value::Int(i) = index_val {
                                i as usize
                            } else if let Value::Number(f) = index_val {
                                f as usize
                            } else {
                                return Err(RuntimeError {
                                    message: "Array index must be an Int or Number".to_string(),
                                    stack_trace: self.generate_stack_trace(&task, &frame),
                                });
                            };

                            if let Value::Array(arr) = obj_val {
                                let arr = arr.lock().unwrap();
                                if idx < arr.len() {
                                    task.registers[base + dest] = arr[idx].clone();
                                } else {
                                    return Err(RuntimeError {
                                        message: format!("Array index out of bounds: {} >= {}", idx, arr.len()),
                                        stack_trace: self.generate_stack_trace(&task, &frame),
                                    });
                                }
                            } else {
                                return Err(RuntimeError {
                                    message: "ArrayIndex on non-array".to_string(),
                                    stack_trace: self.generate_stack_trace(&task, &frame),
                                });
                            }
                        }
                        Opcode::ArrayAssign(obj_reg, index_reg, val_reg) => {
                            let val = task.registers[base + val_reg].clone();
                            let index_val = task.registers[base + index_reg].clone();
                            
                            let idx = if let Value::Int(i) = index_val {
                                i as usize
                            } else if let Value::Number(f) = index_val {
                                f as usize
                            } else {
                                return Err(RuntimeError {
                                    message: "Array index must be an Int or Number".to_string(),
                                    stack_trace: self.generate_stack_trace(&task, &frame),
                                });
                            };

                            let obj_val = task.registers[base + obj_reg].clone();
                            if let Value::Array(arr) = obj_val {
                                let mut arr = arr.lock().unwrap();
                                if idx < arr.len() {
                                    arr[idx] = val;
                                } else {
                                    return Err(RuntimeError {
                                        message: format!("Array index out of bounds: {} >= {}", idx, arr.len()),
                                        stack_trace: self.generate_stack_trace(&task, &frame),
                                    });
                                }
                            } else {
                                return Err(RuntimeError {
                                    message: "ArrayAssign on non-array".to_string(),
                                    stack_trace: self.generate_stack_trace(&task, &frame),
                                });
                            }
                        }
                        Opcode::MakeEnum(dest, variant_idx, start_reg, count) => {
                            let mut values = Vec::new();
                            for i in 0..count {
                                values.push(task.registers[base + start_reg + i].clone());
                            }
                            task.registers[base + dest] = Value::Enum(variant_idx, values);
                        }
                        Opcode::CheckEnum(dest, obj, expected_variant) => {
                            let obj_val = &task.registers[base + obj];
                            if let Value::Enum(actual_variant, _) = obj_val {
                                task.registers[base + dest] = Value::Bool(*actual_variant == expected_variant);
                            } else {
                                task.registers[base + dest] = Value::Bool(false);
                            }
                        }
                        Opcode::ExtractEnum(dest_start, obj, count) => {
                            let obj_val = task.registers[base + obj].clone();
                            if let Value::Enum(_, values) = obj_val {
                                if values.len() != count {
                                    return Err(RuntimeError {
                                        message: format!("ExtractEnum expected {} values, found {}", count, values.len()),
                                        stack_trace: self.generate_stack_trace(&task, &frame),
                                    });
                                }
                                for (i, v) in values.iter().enumerate() {
                                    task.registers[base + dest_start + i] = v.clone();
                                }
                            } else {
                                return Err(RuntimeError {
                                    message: "ExtractEnum on invalid enum".to_string(),
                                    stack_trace: self.generate_stack_trace(&task, &frame),
                                });
                            }
                        }
                        Opcode::TryUnwrap(dest, src) => {
                            let result_val = task.registers[base + src].clone();
                            if let Value::Enum(variant_idx, values) = result_val {
                                if variant_idx == 0 { // Ok
                                    task.registers[base + dest] = values[0].clone();
                                } else { // Err
                                    try_return_val = Some(Value::Enum(variant_idx, values));
                                    break;
                                }
                            } else {
                                return Err(RuntimeError {
                                    message: "TryUnwrap expected Result".to_string(),
                                    stack_trace: self.generate_stack_trace(&task, &frame),
                                });
                            }
                        }
                    }
                }
                
                if yielded {
                    break;
                }

                // If we exit the inner loop without yielding, we either finished the chunk or hit Return
                // Wait, if we hit Return, we need to extract the return value.
                // Let's get the last executed instruction.
                let mut ret_val = Value::Null;
                if let Some(val) = try_return_val {
                    ret_val = val;
                } else if frame.ip > 0 {
                    if let Opcode::Return(src) = frame.chunk.instructions[frame.ip - 1] {
                        ret_val = task.registers[frame.base_register + src].clone();
                    }
                }

                if let Some(caller_frame) = task.frames.pop() {
                    task.registers[frame.dest_register] = ret_val;
                    task.frames.push(caller_frame);
                } else {
                    task.result = Some(ret_val);
                    break;
                }
            }

            if task.result.is_none() {
                self.tasks.insert(task_id, task);
                self.ready_queue.push_back(task_id);
            } else {
                if task.id == 0 {
                    return Ok(());
                } else {
                    self.tasks.insert(task_id, task);
                }
            }
        }
        
        Ok(())
    }
}
