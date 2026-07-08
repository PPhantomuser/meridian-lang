# The Meridian Programming Language: A Comprehensive Guide (The "Bible")

Welcome to the definitive guide for **Meridian**! Whether you are a seasoned developer coming from Python, Rust, Swift, or C++, or you are an AI agent looking to understand the core semantics of the language, this guide will serve as your complete reference.

Meridian is a serious, compiled-first programming language designed from the ground up for the era of Human-AI Pair Programming. After reading this, you will have all the knowledge necessary to build any project, logic, or system using Meridian.

---

## Chapter 1: Vision & Philosophy

Meridian was born out of a need for a language that bridges the gap between high-level ergonomics (like Python) and low-level guarantees (like Rust), while maintaining **100% predictable AI legibility**. 

### Key Tenets
1. **Statically Typed**: Types are explicit and strictly checked. If it compiles, it works.
2. **Compiled First**: Meridian runs on a custom Virtual Machine (`meridian_vm`) and can be natively JIT-compiled via Cranelift for maximum performance.
3. **AI Legible**: The compiler's AST and diagnostics are deterministically serializable to JSON. This means AI agents can read the *exact machine state* of your code, ensuring they don't hallucinate bugs.
4. **Expression-Based**: Almost everything in Meridian is an expression that evaluates to a value.

---

## Chapter 2: Toolchain & Usage

Meridian ships as a unified, zero-configuration binary called `merid`. You don't need a complex build setup to get started.

### Running Code
To run a script (`.mer` file) through the Virtual Machine:
```bash
cargo run -p meridian_cli -- run path/to/script.mer
```

To compile and run natively using Cranelift (for maximum speed):
```bash
cargo run -p meridian_cli -- run --release path/to/script.mer
```

### Static Analysis & Type Checking
If you want to verify your code without running it (highly recommended for AI agents before proposing code):
```bash
cargo run -p meridian_cli -- check path/to/script.mer
```
*(If errors exist, the CLI will output a structured JSON array of diagnostics, pinpointing the exact byte offsets and issues.)*

### Formatting & Linting
Meridian uses a built-in formatter and linter. All code must adhere to standard formatting to reduce cognitive load:
```bash
cargo run -p meridian_cli -- fmt path/to/script.mer
cargo run -p meridian_cli -- lint path/to/script.mer
```

---

## Chapter 3: Syntax & Basic Concepts

### 3.1 Variables & Mutability
By default, variables are **immutable** (they cannot be changed once assigned). This prevents a whole class of state-related bugs. To make a variable mutable, explicitly use `let mut`.

```meridian
// Variables require explicit type annotations.
let pi: Number = 3.14159;
let message: String = "Hello, Meridian!";
let is_active: Bool = true;

// Mutable variable
let mut counter: Number = 0;
counter = counter + 1; // Valid because of `mut`
```

### 3.2 Primitive Types
Meridian keeps primitives simple and powerful:
- `Number`: A 64-bit floating point number (`f64`), used for both integers and decimals.
- `String`: A standard UTF-8 string literal.
- `Bool`: `true` or `false`.
- `Unit`: The empty return type (similar to `()` in Rust or `void` in C/C++).

### 3.3 Arithmetic & Logic
Standard operators apply: `+`, `-`, `*`, `/`. Equality is checked using `==`.
```meridian
let a: Number = (5 * 2) + 1;
let is_eleven: Bool = a == 11;
```

---

## Chapter 4: Control Flow

Meridian is **expression-based**. This means constructs like `if` / `else` evaluate to a value and can be assigned directly to variables.

### `if` Expressions
There is no `return` keyword needed inside control flow blocks. The last expression in the block is automatically yielded as the value.

```meridian
let state: Number = if x == 10 {
    1
} else if x == 20 {
    2
} else {
    0
};
```
> [!IMPORTANT]
> **Rule of Thumb:** If an `if` expression returns a value, it MUST have an `else` branch to guarantee a value is produced in all paths.

---

## Chapter 5: Functions

Functions are defined using the `fn` keyword. They require explicit type annotations for parameters and the return type.

### Defining and Calling Functions
Notice that there is **no `return` keyword**. The final expression in the function body is implicitly returned.

```meridian
fn add_numbers(a: Number, b: Number) -> Number {
    a + b // Implicit return!
}

// A more complex example using recursion
fn collatz_steps(n: Number) -> Number {
    if n == 1 {
        0
    } else {
        if is_even(n) == 1 {
            let next_n = n / 2;
            1 + collatz_steps(next_n)
        } else {
            let next_n = 3 * n + 1;
            1 + collatz_steps(next_n)
        }
    }
}
```

---

## Chapter 6: Memory & Borrowing

Meridian ensures absolute memory safety at compile time using a **lexical borrow checker** (similar to Rust, but designed for ergonomics). 

### References
Instead of copying large data around, you can "borrow" it.
- **Shared Reference (`&T`)**: Read-only. You can have many of these at once.
- **Mutable Reference (`&mut T`)**: Read-write. You can only have **one** of these at a time, and no shared references can exist simultaneously.

### The Golden Rules of Borrowing
1. **No Mutable Aliases**: If a variable is borrowed mutably (`&mut x`), it cannot be borrowed again (mutably or immutably) in the same block.
2. **Multiple Readers**: You can have multiple shared immutable borrows (`&x`) active at the same time.
3. **Immutability First**: You cannot mutably borrow a variable that was declared as immutable (`let`).

```meridian
fn modify_value(val: &mut Number) -> Unit {
    *val = *val + 10; // Use * to dereference and modify the value
}

fn main() -> Unit {
    let mut x: Number = 5;
    modify_value(&mut x);
    print x; // Output: 15
}
```

> [!TIP]
> **Lifetime Elision:** Meridian forbids explicit lifetimes (like `<'a>` in Rust). If a function returns a reference, it MUST take exactly ONE reference parameter. This keeps the code clean and AI-friendly.

---

## Chapter 7: Advanced Data Structures

Meridian moves away from classical Object-Oriented inheritance (which is fragile) and favors **composition** and **traits**.

### Structs
Used to group related data together.
```meridian
struct Point {
    x: Number,
    y: Number,
}
```

### Enums (Sum Types)
Enums in Meridian are powerful—they can contain payloads, allowing you to represent different states safely.
```meridian
enum NetworkState {
    Disconnected,
    Connecting(String), // Contains the URL
    Connected(Number),  // Contains ping latency
}
```

### Traits
Traits define shared behavior (interfaces) that types can implement.
```meridian
trait Drawable {
    fn draw(&self) -> Unit;
}
```

---

## Chapter 8: Pattern Matching & Error Handling

Meridian makes illegal states unrepresentable. Error handling relies on `Result` and `Option` types rather than exceptions or silent failures.

### Pattern Matching (`match`)
The `match` statement ensures that you handle every possible case exhaustively.
```meridian
let state = NetworkState::Connecting("https://api.meridian.dev");

match state {
    NetworkState::Disconnected => print "Offline",
    NetworkState::Connecting(url) => {
        // url is safely extracted here
        print "Connecting to...";
    },
    NetworkState::Connected(ping) => print "Online!",
}
```

### Error Handling (`Result` and `Option`)
Instead of throwing exceptions, functions that can fail return a `Result<T, E>`.
- `T`: The success type.
- `E`: The error type.

You can propagate errors easily using the `?` operator.
```meridian
fn read_config(path: String) -> Result<String, IOError> {
    let content = read_file(path)?; // If it fails, returns the error immediately
    Ok(content)
}
```

---

## Chapter 9: The AI-Native Advantage

If you are a human, you'll love Meridian for its speed, safety, and modern syntax.
If you are an AI, you'll love Meridian because:
1. **No Token Ambiguity**: Keyword-heavy syntax (`fn`, `let`, `mut`, `match`) means the language is easily tokenized and understood by LLMs.
2. **Structured Errors**: The `merid check --json` output lets AIs programmatically understand what went wrong and fix it without scraping string text.
3. **CST-Preserving AST**: AIs can diff and patch code at the AST structural level, avoiding indentation and whitespace hallucinations.

---

## Conclusion
You are now equipped to write, analyze, and build robust software in Meridian. Focus on expression-based logic, respect the borrow checker, and leverage pattern matching. Happy coding!

---

## Appendix: 10 Practical Examples in Meridian

This appendix provides 10 real-world examples of Meridian code, ranging from basic syntax to advanced memory safety and networking. You can copy these into a `.mer` file and run them using `cargo run -p meridian_cli -- run <file.mer>`.

### Example 1: Hello World
**Problem Statement:** You want to print a simple message to the console.
**Implementation:**
```meridian
fn main() -> Unit {
    let message: String = "Hello, World!";
    print message;
}
```

### Example 2: Basic Mathematics & Variables
**Problem Statement:** You need to calculate the area of a rectangle using variables.
**Implementation:**
```meridian
fn main() -> Unit {
    let width: Number = 10;
    let height: Number = 20;
    let area: Number = width * height;
    
    print "The area is:";
    print area;
}
```

### Example 3: Expression-Based Control Flow (Even or Odd)
**Problem Statement:** Determine if a number is even or odd without using the `return` keyword.
**Implementation:**
```meridian
fn is_even(n: Number) -> Bool {
    // Note: No return keyword. The `if` expression yields the value.
    if n % 2 == 0 {
        true
    } else {
        false
    }
}

fn main() -> Unit {
    let result: Bool = is_even(42);
    print result; // Output: true
}
```

### Example 4: Recursion (The Fibonacci Sequence)
**Problem Statement:** Calculate the nth Fibonacci number recursively.
**Implementation:**
```meridian
fn fibonacci(n: Number) -> Number {
    if n == 0 {
        0
    } else if n == 1 {
        1
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

fn main() -> Unit {
    let fib_10: Number = fibonacci(10);
    print fib_10; // Output: 55
}
```

### Example 5: The Borrow Checker (Mutable References)
**Problem Statement:** You want to safely modify an existing variable's value inside a function without returning a new one.
**Implementation:**
```meridian
fn double_value(val: &mut Number) -> Unit {
    // Dereference with * to mutate the underlying value safely
    *val = *val * 2;
}

fn main() -> Unit {
    let mut score: Number = 15;
    double_value(&mut score);
    print score; // Output: 30
}
```

### Example 6: Custom Types (Structs)
**Problem Statement:** You need to group coordinates into a single data structure.
**Implementation:**
```meridian
struct Point {
    x: Number,
    y: Number,
}

fn print_point(p: &Point) -> Unit {
    print "Point X:";
    print p.x;
}

fn main() -> Unit {
    let origin = Point { x: 0, y: 0 };
    print_point(&origin);
}
```

### Example 7: Safe State Management (Enums & Pattern Matching)
**Problem Statement:** You want to model the state of a payment, ensuring you handle every possible scenario safely.
**Implementation:**
```meridian
enum PaymentStatus {
    Pending,
    Failed(String), // Payload contains failure reason
    Success(Number), // Payload contains transaction ID
}

fn handle_payment(status: PaymentStatus) -> Unit {
    match status {
        PaymentStatus::Pending => print "Waiting for payment...",
        PaymentStatus::Failed(reason) => {
            print "Payment failed due to:";
            print reason;
        },
        PaymentStatus::Success(tx_id) => {
            print "Payment successful! ID:";
            print tx_id;
        },
    }
}

fn main() -> Unit {
    let status = PaymentStatus::Failed("Insufficient funds");
    handle_payment(status);
}
```

### Example 8: Safe Error Handling (Result and Option)
**Problem Statement:** Read a file from disk, but handle the case where the file doesn't exist without crashing the program.
**Implementation:**
```meridian
fn read_config() -> Result<String, String> {
    // The `?` operator automatically returns an error if `read_file` fails
    let content = read_file("config.json")?;
    Ok(content)
}

fn main() -> Unit {
    let result = read_config();
    match result {
        Ok(data) => print data,
        Err(e) => print "Failed to read config!",
    }
}
```

### Example 9: Data Processing (JSON & HashMaps)
**Problem Statement:** Parse a JSON string into a native object and read data from it.
**Implementation:**
```meridian
fn process_payload() -> Unit {
    let raw_json: String = "{\"user\": \"alice\", \"role\": \"admin\"}";
    
    // Using the standard library JSON parser
    let parsed_data = json_parse(raw_json);
    
    // Normally you would extract and match fields here based on your HashMap
    print json_stringify(parsed_data);
}

fn main() -> Unit {
    process_payload();
}
```

### Example 10: Networking (A Simple TCP Server)
**Problem Statement:** Start a server that listens on a port, accepts a connection, and reads data sent by the client.
**Implementation:**
```meridian
fn start_server() -> Unit {
    // Bind the listener to port 8080
    let listener = tcp_bind("127.0.0.1:8080");
    print "Server running on port 8080...";
    
    // Wait for a client connection
    let stream = tcp_accept(listener);
    print "Client connected!";
    
    // Read the message from the client
    let message = tcp_read(stream);
    print "Client says:";
    print message;
}

fn main() -> Unit {
    start_server();
}
```

### Example 11: Reading and Parsing a Configuration File
**Problem Statement:** You need to load a configuration file from disk, parse it as JSON, and handle potential errors if the file doesn't exist or is invalid.
**Implementation:**
```meridian
fn load_config(path: String) -> Result<String, String> {
    // 1. Read the file. If it fails, bubble up the error using `?`
    let raw_content = read_file(path)?;
    
    // 2. Parse the JSON. 
    let config_obj = json_parse(raw_content);
    
    // 3. Stringify it back out for demonstration
    let config_string = json_stringify(config_obj);
    
    Ok(config_string)
}

fn main() -> Unit {
    let result = load_config("settings.json");
    match result {
        Ok(data) => print data,
        Err(e) => print "Could not load settings.json!",
    }
}
```

### Example 12: Implementing Polymorphism with Traits
**Problem Statement:** You want to define a common interface for different shapes to calculate their area, ensuring consistent behavior.
**Implementation:**
```meridian
trait Area {
    fn calculate_area(&self) -> Number;
}

struct Circle {
    radius: Number,
}

// In Meridian, you implement traits for specific structs
impl Area for Circle {
    fn calculate_area(&self) -> Number {
        self.radius * self.radius * 3.14159
    }
}

struct Rectangle {
    w: Number,
    h: Number,
}

impl Area for Rectangle {
    fn calculate_area(&self) -> Number {
        self.w * self.h
    }
}

fn print_area(shape: &Area) -> Unit {
    print "The area is:";
    print shape.calculate_area();
}
```

### Example 13: Executing System Subprocesses
**Problem Statement:** You want to execute a terminal command (like `ls` or `curl`) from within your Meridian code and capture the output.
**Implementation:**
```meridian
fn fetch_website() -> Unit {
    // Create a new queue for command arguments
    let args = vecdeque_new();
    vecdeque_push_back(args, "https://api.github.com");
    
    // Execute the `curl` command using the stdlib process_output
    let output = process_output("curl", args);
    
    print "Command Output:";
    print output;
}

fn main() -> Unit {
    fetch_website();
}
```

### Example 14: Simulating a State Machine
**Problem Statement:** Model a vending machine's internal state to ensure it cannot dispense items if no money has been inserted.
**Implementation:**
```meridian
enum VendingState {
    Idle,
    CoinInserted(Number), // Payload is the amount
    Dispensing(String),   // Payload is the item name
}

fn process_state(current_state: VendingState) -> VendingState {
    match current_state {
        VendingState::Idle => {
            print "Waiting for coins...";
            VendingState::Idle
        },
        VendingState::CoinInserted(amount) => {
            if amount == 5 {
                print "Enough money! Dispensing...";
                VendingState::Dispensing("Soda")
            } else {
                print "Not enough money.";
                VendingState::Idle
            }
        },
        VendingState::Dispensing(item) => {
            print "Here is your:";
            print item;
            VendingState::Idle
        },
    }
}
```

### Example 15: Managing Collections (HashMaps)
**Problem Statement:** You need to store key-value pairs (like user sessions) and safely retrieve them.
**Implementation:**
```meridian
fn manage_sessions() -> Unit {
    // Create a new map
    let sessions = hashmap_new();
    
    // Insert values
    hashmap_insert(sessions, "user_123", "active");
    hashmap_insert(sessions, "user_456", "inactive");
    
    // Retrieve values safely
    let status = hashmap_get(sessions, "user_123");
    print "User 123 status is:";
    print status;
}

fn main() -> Unit {
    manage_sessions();
}
```

### Example 16: Building a File Logger
**Problem Statement:** Create a system that appends log messages to a file without overwriting the previous contents.
**Implementation:**
```meridian
fn log_message(message: String) -> Unit {
    let path = "server.log";
    
    // Check if file exists, if not, create it by writing. Otherwise, append.
    if file_exists(path) {
        file_append(path, message);
        file_append(path, "\n");
    } else {
        file_write(path, message);
        file_append(path, "\n");
    }
}

fn main() -> Unit {
    log_message("SERVER STARTING...");
    log_message("Warning: High memory usage.");
    print "Logs written.";
}
```

### Example 17: Mutable References with Collections
**Problem Statement:** You have a queue (VecDeque) of tasks and you want to pass it mutably to a worker function to process and empty it.
**Implementation:**
```meridian
fn process_queue(queue: &mut NativeObject) -> Unit {
    let task1 = vecdeque_pop_front(queue);
    print "Processed task:";
    print task1;
    
    let task2 = vecdeque_pop_front(queue);
    print "Processed task:";
    print task2;
}

fn main() -> Unit {
    let mut task_queue = vecdeque_new();
    vecdeque_push_back(task_queue, "Task A");
    vecdeque_push_back(task_queue, "Task B");
    
    // Safely borrow the queue mutably
    process_queue(&mut task_queue);
}
```

### Example 18: Advanced Pattern Matching (Nested Enums)
**Problem Statement:** You are processing network packets where the packet itself has different versions and internal payload structures.
**Implementation:**
```meridian
enum Protocol {
    IPv4(String),
    IPv6(String),
}

enum Packet {
    Ping(Protocol),
    Data(Protocol, String),
}

fn analyze_packet(packet: Packet) -> Unit {
    match packet {
        Packet::Ping(Protocol::IPv4(ip)) => {
            print "Ping received from IPv4:";
            print ip;
        },
        Packet::Ping(Protocol::IPv6(ip)) => {
            print "Ping received from IPv6:";
            print ip;
        },
        Packet::Data(_, payload) => {
            print "Data packet received with payload:";
            print payload;
        },
    }
}
```

### Example 19: An HTTP-Like Request Handler
**Problem Statement:** Build a function that reads from a TCP stream and responds with a basic HTTP 200 OK message.
**Implementation:**
```meridian
fn handle_connection(stream: NativeObject) -> Unit {
    // Read the incoming request
    let request = tcp_read(stream);
    print "Incoming Request:";
    print request;
    
    // Format a valid HTTP response
    let response = "HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello, World!";
    
    // Write back to the client
    tcp_write(stream, response);
}

fn main() -> Unit {
    let listener = tcp_bind("0.0.0.0:8080");
    let stream = tcp_accept(listener);
    handle_connection(stream);
}
```

### Example 20: Safe Division with Option Types
**Problem Statement:** Perform a mathematical division but prevent crashes caused by dividing by zero using an `Option` type.
**Implementation:**
```meridian
enum OptionNumber {
    Some(Number),
    None,
}

fn safe_divide(numerator: Number, denominator: Number) -> OptionNumber {
    if denominator == 0 {
        OptionNumber::None
    } else {
        OptionNumber::Some(numerator / denominator)
    }
}

fn main() -> Unit {
    let result = safe_divide(100, 0);
    
    match result {
        OptionNumber::Some(val) => {
            print "The answer is:";
            print val;
        },
        OptionNumber::None => {
            print "Error: Cannot divide by zero!";
        },
    }
}
```

---

## Appendix 2: Deep Dive into Meridian Systems (12 Advanced Examples)

This section focuses heavily on Meridian's equivalents for Object-Oriented Programming (OOP), Pointers (Borrowing), File System operations, Advanced Error Handling, and the Standard Template Library (Collections).

### Example 21: OOP - Composition (Struct inside Struct)
**Problem Statement:** Meridian prefers Composition over Inheritance. You need to model a complex entity (a Player) that owns other data structures (Inventory and Stats).
**Implementation:**
```meridian
struct Stats {
    health: Number,
    mana: Number,
}

struct Inventory {
    gold: Number,
    items_count: Number,
}

struct Player {
    name: String,
    stats: Stats,
    inventory: Inventory,
}

fn main() -> Unit {
    let hero = Player {
        name: "Arthur",
        stats: Stats { health: 100, mana: 50 },
        inventory: Inventory { gold: 500, items_count: 3 },
    };
    
    print "Hero Name:";
    print hero.name;
    print "Hero Health:";
    print hero.stats.health;
}
```

### Example 22: OOP - Advanced Traits (Interfaces)
**Problem Statement:** You want multiple types of logging systems (Console and File) that conform to the same interface, proving polymorphism.
**Implementation:**
```meridian
trait Logger {
    fn log(&self, msg: String) -> Unit;
}

struct ConsoleLogger {
    prefix: String,
}

impl Logger for ConsoleLogger {
    fn log(&self, msg: String) -> Unit {
        print self.prefix;
        print msg;
    }
}

struct FileLogger {
    file_path: String,
}

impl Logger for FileLogger {
    fn log(&self, msg: String) -> Unit {
        // Appends to file
        file_append(self.file_path, msg);
        file_append(self.file_path, "\n");
    }
}
```

### Example 23: Pointers - Shared Immutable References (`&T`)
**Problem Statement:** You have a massive data structure and you want multiple functions to read it simultaneously without copying it and without risking modifications.
**Implementation:**
```meridian
struct HeavyData {
    id: Number,
    payload: String,
}

// Function takes an IMMUTABLE reference (&)
fn read_id(data: &HeavyData) -> Unit {
    print "ID is:";
    print data.id;
}

// Function takes an IMMUTABLE reference (&)
fn read_payload(data: &HeavyData) -> Unit {
    print "Payload is:";
    print data.payload;
}

fn main() -> Unit {
    let my_data = HeavyData { id: 99, payload: "Gigabytes of text..." };
    
    // We can have multiple active immutable references
    read_id(&my_data);
    read_payload(&my_data);
}
```

### Example 24: Pointers - The Mutable Borrow Checker at Work
**Problem Statement:** Prove that Meridian prevents "Data Races" by forbidding multiple mutable pointers to the same data at the same time.
**Implementation:**
```meridian
fn mutate_data(data: &mut Number) -> Unit {
    *data = *data + 1;
}

fn main() -> Unit {
    let mut counter: Number = 0;
    
    // This is valid: One mutable borrow
    mutate_data(&mut counter);
    
    // THE COMPILER PREVENTS THIS: 
    // You cannot do:
    // let ptr1 = &mut counter;
    // let ptr2 = &mut counter; 
    // Because two parts of the code could mutate `counter` unpredictably!
    
    print counter; // Output: 1
}
```

### Example 25: File Handling - Checking and Deleting
**Problem Statement:** A script that checks if a temporary file exists. If it does, it deletes it. If it doesn't, it creates it.
**Implementation:**
```meridian
fn manage_temp_file() -> Unit {
    let path = "temp_cache.txt";
    
    let exists = file_exists(path);
    if exists {
        print "File exists. Deleting it...";
        file_delete(path);
        print "Deleted.";
    } else {
        print "File does not exist. Creating it...";
        file_write(path, "Cache Initialized");
        print "Created.";
    }
}
```

### Example 26: File Handling - Appending Logs
**Problem Statement:** Safely writing continuous event logs to a text file.
**Implementation:**
```meridian
fn record_event(event_name: String) -> Unit {
    let log_file = "events.log";
    
    // Append the event
    file_append(log_file, "EVENT OCCURRED: ");
    file_append(log_file, event_name);
    file_append(log_file, "\n");
}

fn main() -> Unit {
    record_event("UserLogin");
    record_event("DataSync");
    print "Events recorded.";
}
```

### Example 27: Error Handling - Custom Error Enums
**Problem Statement:** Instead of generic string errors, use a strict Enum to define exactly what can go wrong when authenticating a user.
**Implementation:**
```meridian
enum AuthError {
    UserNotFound,
    IncorrectPassword,
    AccountLocked(Number), // Payload: minutes until unlock
}

fn login(username: String, pass: String) -> Result<String, AuthError> {
    if username == "admin" {
        if pass == "secret123" {
            Ok("Token_998877")
        } else {
            Err(AuthError::IncorrectPassword)
        }
    } else {
        Err(AuthError::UserNotFound)
    }
}
```

### Example 28: Error Handling - Safe Unwrapping with `match`
**Problem Statement:** Call a function that returns a `Result` and exhaustively handle every single error subtype.
**Implementation:**
```meridian
fn main() -> Unit {
    let login_attempt = login("admin", "wrong_pass");
    
    match login_attempt {
        Ok(token) => {
            print "Login Success! Token:";
            print token;
        },
        Err(AuthError::UserNotFound) => print "Error: No such user.",
        Err(AuthError::IncorrectPassword) => print "Error: Wrong password.",
        Err(AuthError::AccountLocked(mins)) => {
            print "Account locked for minutes:";
            print mins;
        },
    }
}
```

### Example 29: STL - Advanced HashMap (Dictionaries/Maps)
**Problem Statement:** Map usernames to their permission levels. Check if a user exists in the map before taking action.
**Implementation:**
```meridian
fn main() -> Unit {
    let permissions = hashmap_new();
    
    // Insert Key-Value pairs (String -> String)
    hashmap_insert(permissions, "alice", "admin");
    hashmap_insert(permissions, "bob", "editor");
    
    // Retrieve a value. Returns Null if not found.
    let role = hashmap_get(permissions, "alice");
    print "Alice's role is:";
    print role;
}
```

### Example 30: STL - HashSet (Unique Elements)
**Problem Statement:** Store a list of banned IP addresses, ensuring there are no duplicates, and quickly check if an incoming IP is banned.
**Implementation:**
```meridian
fn firewall_check(ip: String) -> Unit {
    let banned_ips = hashset_new();
    
    // Adding duplicates doesn't matter in a HashSet
    hashset_insert(banned_ips, "192.168.1.100");
    hashset_insert(banned_ips, "10.0.0.5");
    hashset_insert(banned_ips, "192.168.1.100"); 
    
    let is_banned = hashset_contains(banned_ips, ip);
    
    if is_banned {
        print "CONNECTION BLOCKED";
    } else {
        print "CONNECTION ALLOWED";
    }
}

fn main() -> Unit {
    firewall_check("10.0.0.5"); // Will be blocked
}
```

### Example 31: STL - VecDeque (Sliding Window / Buffer)
**Problem Statement:** You are receiving a stream of data and only want to keep the 3 most recent messages (a sliding window buffer).
**Implementation:**
```meridian
fn sliding_window() -> Unit {
    let buffer = vecdeque_new();
    
    // Push 4 messages to the back
    vecdeque_push_back(buffer, "Msg 1");
    vecdeque_push_back(buffer, "Msg 2");
    vecdeque_push_back(buffer, "Msg 3");
    vecdeque_push_back(buffer, "Msg 4");
    
    // Pop from the front to maintain a size of 3
    let oldest = vecdeque_pop_front(buffer);
    print "Dropped oldest message:";
    print oldest; // Outputs: Msg 1
}
```

### Example 32: Advanced Data Structures - Recursive Enums (Linked Lists)
**Problem Statement:** Build a linked list using Meridian's recursive Enum capabilities (Wait, memory sizes for recursive enums require indirection like `Box` in Rust, but in Meridian's AST abstraction, it can be modeled cleanly).
**Implementation:**
```meridian
enum LinkedList {
    Empty,
    // A Node contains a Number and a pointer to the next LinkedList
    Node(Number, &LinkedList), 
}

fn print_list(list: &LinkedList) -> Unit {
    match list {
        LinkedList::Empty => print "End of list",
        LinkedList::Node(value, next_node) => {
            print "Node value:";
            print value;
            // Recursive call to print the next node
            print_list(next_node);
        },
    }
}
```
