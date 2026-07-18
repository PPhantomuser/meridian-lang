#![allow(unused)]
use clap::{Parser as ClapParser, Subcommand};
use meridian_lexer::Lexer;
use meridian_parser::Parser;
use meridian_semantic::SemanticAnalyzer;
use meridian_ir::Compiler;
use meridian_vm::VM;
use std::fs;
use std::collections::{HashSet, HashMap};
use std::path::{Path, PathBuf};
use meridian_diagnostics::{Diagnostic, DiagnosticCategory, Span};
use meridian_ast::{Program, Stmt};

mod resolver;




#[derive(ClapParser)]
#[command(name = "merid")]
#[command(about = "Meridian compiler and toolchain", long_about = None)]
struct Cli {
    #[arg(long, global = true)]
    ai_diagnostics: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run { 
        file: Option<String>,
        #[arg(long)]
        release: bool,
        #[arg(long)]
        allow_net: bool,
        #[arg(long)]
        allow_fs: bool,
        #[arg(long)]
        allow_run: bool,
        #[arg(long)]
        allow_all: bool,
    },

    Ast { file: String },
    Check { 
        file: String,
        #[arg(long)]
        allow_net: bool,
        #[arg(long)]
        allow_fs: bool,
        #[arg(long)]
        allow_run: bool,
        #[arg(long)]
        allow_all: bool,
    },
    Fmt { file: String },
    Lint { file: String },
    Test { 
        file: Option<String>,
        #[arg(long)]
        workspace: bool,
        #[arg(long)]
        allow_net: bool,
        #[arg(long)]
        allow_fs: bool,
        #[arg(long)]
        allow_run: bool,
        #[arg(long)]
        allow_all: bool,
    },
    Build {
        file: Option<String>,
        #[arg(long)]
        allow_net: bool,
        #[arg(long)]
        allow_fs: bool,
        #[arg(long)]
        allow_run: bool,
        #[arg(long)]
        allow_all: bool,
    },
    Audit,
    Lsp,
    Pkg {
        #[command(subcommand)]
        cmd: PkgCommands,
    },
}

#[derive(Subcommand)]
pub enum PkgCommands {
    Fetch,
    Search { name: String },
    Publish { source: String, commit: String },
}

fn print_diagnostics(diagnostics: &[Diagnostic], ai_mode: bool) {
    if ai_mode {
        let json = serde_json::to_string_pretty(&diagnostics).unwrap();
        eprintln!("{}", json);
    } else {
        for diag in diagnostics {
            let cat = match diag.category {
                DiagnosticCategory::Lexical => "Lexical Error",
                DiagnosticCategory::Syntax => "Syntax Error",
                DiagnosticCategory::Semantic => "Semantic Error",
                DiagnosticCategory::Type => "Type Error",
                DiagnosticCategory::Lint => "Lint Warning",
            };
            eprintln!("{}: [{}] {}", cat, diag.machine_code, diag.message);
            if let Some(sugg) = &diag.suggestion {
                eprintln!("  Suggestion: {}", sugg);
            }
        }
    }
}

fn load_module(
    file_path: &Path,
    visited: &mut HashSet<PathBuf>,
    diagnostics: &mut Vec<Diagnostic>,
    resolved_paths: Option<&HashMap<String, PathBuf>>,
) -> Option<Program> {
    let canonical_path = match std::fs::canonicalize(file_path) {
        Ok(p) => p,
        Err(_) => {
            diagnostics.push(Diagnostic::new(
                format!("Failed to resolve file: {}", file_path.display()),
                "MER0201".to_string(),
                Span::new(0, 0),
                DiagnosticCategory::Syntax,
                None,
            ));
            return None;
        }
    };

    if visited.contains(&canonical_path) {
        diagnostics.push(Diagnostic::new(
            format!("Cyclic dependency detected: {}", canonical_path.display()),
            "MER0200".to_string(),
            Span::new(0, 0),
            DiagnosticCategory::Semantic,
            None,
        ));
        return None;
    }

    visited.insert(canonical_path.clone());

    let source = match fs::read_to_string(&canonical_path) {
        Ok(s) => s,
        Err(_) => {
            diagnostics.push(Diagnostic::new(
                format!("Failed to read file: {}", canonical_path.display()),
                "MER0202".to_string(),
                Span::new(0, 0),
                DiagnosticCategory::Syntax,
                None,
            ));
            return None;
        }
    };

    let lexer = Lexer::new(&source);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();

    for d in parser.diagnostics {
        diagnostics.push(d);
    }

    let mut flattened_statements = Vec::new();
    let parent_dir = canonical_path.parent().unwrap_or_else(|| Path::new(""));

    for stmt in program.statements {
        if let Stmt::Import(import_path, span) = stmt {
            let mut next_path;
            
            if import_path.starts_with("http://") || import_path.starts_with("https://") {
                let cache_dir = std::env::temp_dir().join("meridian_cache");
                let _ = std::fs::create_dir_all(&cache_dir);
                let hash = {
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};
                    let mut hasher = DefaultHasher::new();
                    import_path.hash(&mut hasher);
                    hasher.finish()
                };
                let cached_file = cache_dir.join(format!("{:016x}.mr", hash));
                if !cached_file.exists() {
                    match ureq::get(&import_path).call() {
                        Ok(response) => {
                            if let Ok(text) = response.into_string() {
                                let _ = std::fs::write(&cached_file, text);
                            }
                        }
                        Err(e) => {
                            diagnostics.push(Diagnostic::new(
                                format!("Failed to fetch URL '{}': {}", import_path, e),
                                "MER0205".to_string(),
                                span,
                                DiagnosticCategory::Semantic,
                                None,
                            ));
                            continue;
                        }
                    }
                }
                next_path = cached_file;
            } else {
                next_path = parent_dir.join(&import_path);
                
                // Check if it's a dependency from Meridian.toml
                if !next_path.exists() {
                    if let Some(paths) = resolved_paths {
                        if let Some(dep_path) = paths.get(&import_path) {
                            let lib_path = dep_path.join("src").join("lib.mr");
                            let main_path = dep_path.join("src").join("main.mr");
                            if lib_path.exists() {
                                next_path = lib_path;
                            } else if main_path.exists() {
                                next_path = main_path;
                            } else {
                                diagnostics.push(Diagnostic::new(
                                    format!("Could not find src/lib.mr or src/main.mr in dependency '{}'", import_path),
                                    "MER0203".to_string(),
                                    span,
                                    DiagnosticCategory::Semantic,
                                    None,
                                ));
                            }
                        }
                    }
                }
            }

            if let Some(mut imported_program) = load_module(&next_path, visited, diagnostics, resolved_paths) {
                flattened_statements.append(&mut imported_program.statements);
            }
        } else {
            flattened_statements.push(stmt);
        }
    }

    Some(Program { statements: flattened_statements })
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Audit => {
            let current_dir = std::env::current_dir().unwrap();
            let toml_path = current_dir.join("Meridian.toml");
            if !toml_path.exists() {
                eprintln!("Error: Meridian.toml not found");
                std::process::exit(1);
            }
            let toml_str = fs::read_to_string(&toml_path).unwrap();
            let manifest: resolver::manifest::MeridianManifest = toml::from_str(&toml_str).unwrap_or_else(|e| {
                eprintln!("Failed to parse Meridian.toml: {}", e);
                std::process::exit(1);
            });
            
            let result = resolver::solve(&current_dir, &manifest, false);
            
            let mut failed = false;
            for diag in result.diagnostics {
                match diag {
                    resolver::diagnostics::ResolverDiagnostic::ChecksumMismatch { package, expected, actual } => {
                        println!("[FAIL] {} (expected: {}, actual: {})", package, expected, actual);
                        failed = true;
                    }
                    _ => {
                        println!("[FAIL] Dependency error: {:?}", diag);
                        failed = true;
                    }
                }
            }
            if !failed {
                if let Some(deps) = manifest.dependencies {
                    for name in deps.keys() {
                        println!("[PASS] {}", name);
                    }
                }
                println!("Audit passed successfully.");
            } else {
                std::process::exit(1);
            }
        }
        Commands::Lsp => {
            meridian_lsp::run_server();
        }
        Commands::Pkg { cmd } => {
            match cmd {
                PkgCommands::Fetch => {
                    let current_dir = std::env::current_dir().unwrap();
                    let toml_path = current_dir.join("Meridian.toml");
                    if !toml_path.exists() {
                        eprintln!("Error: Meridian.toml not found");
                        std::process::exit(1);
                    }
                    let toml_str = fs::read_to_string(&toml_path).unwrap();
                    let manifest: resolver::manifest::MeridianManifest = toml::from_str(&toml_str).unwrap_or_else(|e| {
                        eprintln!("Failed to parse Meridian.toml: {}", e);
                        std::process::exit(1);
                    });
                    
                    let result = resolver::solve(&current_dir, &manifest, false);
                    let mut has_errors = false;
                    for diag in &result.diagnostics {
                        let json = serde_json::to_string_pretty(diag).unwrap();
                        eprintln!("{}", json);
                        has_errors = true;
                    }
                    if has_errors {
                        std::process::exit(1);
                    }
                    println!("Fetched dependencies successfully.");
                }
                PkgCommands::Search { name } => {
                    if let Some(index) = resolver::registry::get_package_index(name) {
                        println!("Package: {}", index.name);
                        println!("Available Versions:");
                        for v in index.versions {
                            println!("  - {} (source: {})", v.version, v.source);
                        }
                    } else {
                        eprintln!("Package '{}' not found in registry.", name);
                    }
                }
                PkgCommands::Publish { source, commit } => {
                    let current_dir = std::env::current_dir().unwrap();
                    let toml_path = current_dir.join("Meridian.toml");
                    if !toml_path.exists() {
                        eprintln!("Error: Meridian.toml not found in current directory.");
                        std::process::exit(1);
                    }
                    
                    let toml_str = fs::read_to_string(&toml_path).unwrap();
                    let manifest: resolver::manifest::MeridianManifest = toml::from_str(&toml_str).unwrap_or_else(|e| {
                        eprintln!("Error parsing Meridian.toml: {}", e);
                        std::process::exit(1);
                    });
                    
                    let pkg_name = manifest.package.name;
                    let version = manifest.package.version;
                    
                    let mut deps = HashMap::new();
                    if let Some(dependencies) = manifest.dependencies {
                        for (dep_name, dep) in dependencies {
                            if let resolver::manifest::Dependency::Version(ver) = dep {
                                deps.insert(dep_name, ver);
                            }
                        }
                    }
                    
                    let deps_opt = if deps.is_empty() { None } else { Some(deps) };
                    match resolver::registry::generate_publish_payload(&pkg_name, &version, source, commit, deps_opt) {
                        Ok(json) => {
                            println!("Successfully generated registry metadata for {} v{}!\n", pkg_name, version);
                            println!("To publish your package, please submit a Pull Request to:");
                            println!("https://github.com/meridian-lang/registry\n");
                            println!("Add the following content to a file named `index/{}.json`:\n", pkg_name);
                            println!("{}", json);
                        }
                        Err(e) => {
                            eprintln!("Error generating metadata: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
        Commands::Build { file, allow_net, allow_fs, allow_run, allow_all } => {
            let mut visited = HashSet::new();
            let mut diagnostics = Vec::new();
            
            let mut resolved_paths_map = None;
            let mut entry_file = PathBuf::new();

            if let Some(f) = file {
                entry_file = PathBuf::from(f);
            } else {
                let current_dir = std::env::current_dir().unwrap();
                let toml_path = current_dir.join("Meridian.toml");
                if toml_path.exists() {
                    let toml_str = fs::read_to_string(&toml_path).unwrap();
                    let manifest: resolver::manifest::MeridianManifest = toml::from_str(&toml_str).unwrap();
                    let result = resolver::solve(&current_dir, &manifest, false);
                    resolved_paths_map = Some(result.resolved_paths);
                    
                    let main_path = current_dir.join("src").join("main.mr");
                    let lib_path = current_dir.join("src").join("lib.mr");
                    if main_path.exists() {
                        entry_file = main_path;
                    } else if lib_path.exists() {
                        entry_file = lib_path;
                    } else {
                        eprintln!("Error: Neither src/main.mr nor src/lib.mr found in package");
                        std::process::exit(1);
                    }
                } else {
                    eprintln!("Error: No file specified and no Meridian.toml found");
                    std::process::exit(1);
                }
            }

            let program_opt = load_module(&entry_file, &mut visited, &mut diagnostics, resolved_paths_map.as_ref());
            if let Some(mut program) = program_opt {
                if !diagnostics.is_empty() {
                    print_diagnostics(&diagnostics, cli.ai_diagnostics);
                    let has_errors = diagnostics.iter().any(|d| matches!(d.category, DiagnosticCategory::Syntax | DiagnosticCategory::Semantic | DiagnosticCategory::Lexical));
                    if has_errors { std::process::exit(1); }
                }

                let mut analyzer = SemanticAnalyzer::new();
                if *allow_net { analyzer.capabilities.insert("--allow-net".to_string()); }
                if *allow_fs { analyzer.capabilities.insert("--allow-fs".to_string()); }
                if *allow_run { analyzer.capabilities.insert("--allow-run".to_string()); }
                if *allow_all { analyzer.capabilities.insert("--allow-all".to_string()); }
                analyzer.analyze_program(&program);

                if !analyzer.diagnostics.is_empty() {
                    print_diagnostics(&analyzer.diagnostics, cli.ai_diagnostics);
                    std::process::exit(1);
                }

                program.statements.extend(analyzer.get_monomorphized_statements());
                let struct_layouts = analyzer.get_struct_layouts();
                let enum_layouts = analyzer.get_enum_layouts();
                let mut compiler = Compiler::new(analyzer.type_map, analyzer.resolved_names, analyzer.auto_borrows, struct_layouts, enum_layouts);
                let program_ir = compiler.compile(&program);

                let aot = meridian_backend_cranelift::AOTCompiler::new();
                let object_bytes = match aot.compile_to_object(&program_ir) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        eprintln!("AOT Compilation Error: {}", e);
                        std::process::exit(1);
                    }
                };
                let obj_path = entry_file.with_extension("o");
                let bin_path = entry_file.with_extension("");
                let c_runtime_path = entry_file.with_file_name("meridian_runtime.c");
                fs::write(&obj_path, object_bytes).unwrap();
                
                let runtime_c = r#"
#include <stdio.h>
#include <stdint.h>
void print_f64(double val) {
    printf("%g\n", val);
}

void print_i64(int64_t val) {
    printf("%lld\n", (long long)val);
}

int main(int argc, char** argv) {
    extern int meridian_main();
    return meridian_main();
}
"#;
                fs::write(&c_runtime_path, runtime_c).unwrap();
                
                // Link with system cc
                let status = std::process::Command::new("cc")
                    .arg(&obj_path)
                    .arg(&c_runtime_path)
                    .arg("-o")
                    .arg(&bin_path)
                    .status()
                    .unwrap();

                if status.success() {
                    println!("Successfully built executable: {}", bin_path.display());
                } else {
                    eprintln!("Error: Linker failed to create executable");
                    std::process::exit(1);
                }
            } else {
                print_diagnostics(&diagnostics, cli.ai_diagnostics);
                std::process::exit(1);
            }
        }
        Commands::Run { file, release, allow_net, allow_fs, allow_run, allow_all } => {
            let mut visited = HashSet::new();
            let mut diagnostics = Vec::new();
            
            let mut resolved_paths_map = None;
            let mut entry_file = PathBuf::new();

            if let Some(f) = file {
                entry_file = PathBuf::from(f);
            } else {
                // Look for Meridian.toml
                let current_dir = std::env::current_dir().unwrap();
                let toml_path = current_dir.join("Meridian.toml");
                if toml_path.exists() {
                    let toml_str = fs::read_to_string(&toml_path).unwrap();
                    match toml::from_str::<resolver::manifest::MeridianManifest>(&toml_str) {
                        Ok(m) => {
                            let result = resolver::solve(&current_dir, &m, false);
                            let mut has_errors = false;
                            for diag in &result.diagnostics {
                                let json = serde_json::to_string_pretty(diag).unwrap();
                                eprintln!("{}", json);
                                has_errors = true;
                            }
                            if has_errors {
                                std::process::exit(1);
                            }
                            resolved_paths_map = Some(result.resolved_paths);
                            
                            let main_path = current_dir.join("src").join("main.mr");
                            let lib_path = current_dir.join("src").join("lib.mr");
                            if main_path.exists() {
                                entry_file = main_path;
                            } else if lib_path.exists() {
                                entry_file = lib_path;
                            } else {
                                eprintln!("Error: Neither src/main.mr nor src/lib.mr found in package");
                                std::process::exit(1);
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to parse Meridian.toml: {}", e);
                            std::process::exit(1);
                        }
                    }
                } else {
                    eprintln!("Error: No file specified and no Meridian.toml found in current directory");
                    std::process::exit(1);
                }
            }

            let program = load_module(&entry_file, &mut visited, &mut diagnostics, resolved_paths_map.as_ref());
            
            if !diagnostics.is_empty() {
                print_diagnostics(&diagnostics, cli.ai_diagnostics);
                std::process::exit(1);
            }

            let mut program = program.unwrap();

            let mut semantic = SemanticAnalyzer::new();
            if *allow_net { semantic.capabilities.insert("--allow-net".to_string()); }
            if *allow_fs { semantic.capabilities.insert("--allow-fs".to_string()); }
            if *allow_run { semantic.capabilities.insert("--allow-run".to_string()); }
            if *allow_all { semantic.capabilities.insert("--allow-all".to_string()); }
            semantic.analyze_program(&program);
            
            if !semantic.diagnostics.is_empty() {
                print_diagnostics(&semantic.diagnostics, cli.ai_diagnostics);
                std::process::exit(1);
            }

            program.statements.extend(semantic.get_monomorphized_statements());
            let struct_layouts = semantic.get_struct_layouts();
            let enum_layouts = semantic.get_enum_layouts();
            let mut compiler = Compiler::new(semantic.type_map, semantic.resolved_names, semantic.auto_borrows, struct_layouts, enum_layouts);
            let program_ir = compiler.compile(&program);

            if *release {
                let mut jit = meridian_backend_cranelift::JITCompiler::new();
                jit.compile_and_run(&program_ir);
            } else {
                let mut vm = VM::new();
                meridian_stdlib::register_all(&mut vm);
                if let Err(e) = vm.run(&program_ir) {
                    eprintln!("Runtime Error: {}", e.message);
                    eprintln!("Stack Trace:");
                    for (func, ip) in e.stack_trace {
                        eprintln!("  at {} (ip: {})", func, ip);
                    }
                    std::process::exit(1);
                }
            }
        }
        Commands::Ast { file } => {
            let mut visited = HashSet::new();
            let mut diagnostics = Vec::new();
            let program = load_module(Path::new(file), &mut visited, &mut diagnostics, None);
            
            if !diagnostics.is_empty() {
                print_diagnostics(&diagnostics, cli.ai_diagnostics);
                std::process::exit(1);
            }

            let json = serde_json::to_string_pretty(&program.unwrap()).unwrap();
            println!("{}", json);
        }
        Commands::Check { file, allow_net, allow_fs, allow_run, allow_all } => {
            let mut visited = HashSet::new();
            let mut diagnostics = Vec::new();
            let program = load_module(Path::new(file), &mut visited, &mut diagnostics, None);
            
            if !diagnostics.is_empty() {
                print_diagnostics(&diagnostics, cli.ai_diagnostics);
                std::process::exit(1);
            }

            let program = program.unwrap();

            let mut semantic = SemanticAnalyzer::new();
            if *allow_net { semantic.capabilities.insert("--allow-net".to_string()); }
            if *allow_fs { semantic.capabilities.insert("--allow-fs".to_string()); }
            if *allow_run { semantic.capabilities.insert("--allow-run".to_string()); }
            if *allow_all { semantic.capabilities.insert("--allow-all".to_string()); }
            semantic.analyze_program(&program);
            
            if !semantic.diagnostics.is_empty() {
                print_diagnostics(&semantic.diagnostics, cli.ai_diagnostics);
                std::process::exit(1);
            } else {
                println!("[]");
            }
        }
        Commands::Fmt { file } => {
            let source = fs::read_to_string(file).expect("Failed to read file");
            let lexer = Lexer::new(&source);
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program();

            if !parser.diagnostics.is_empty() {
                print_diagnostics(&parser.diagnostics, cli.ai_diagnostics);
                std::process::exit(1);
            }

            let formatter = meridian_fmt::Formatter::new();
            let formatted = formatter.format_program(&program);
            fs::write(file, formatted).expect("Failed to write formatted file");
        }
        Commands::Lint { file } => {
            let mut visited = HashSet::new();
            let mut diagnostics = Vec::new();
            let program = load_module(Path::new(file), &mut visited, &mut diagnostics, None);
            
            if !diagnostics.is_empty() {
                if cli.ai_diagnostics {
                    let json = serde_json::to_string_pretty(&diagnostics).unwrap();
                    println!("{}", json);
                } else {
                    print_diagnostics(&diagnostics, false);
                }
                std::process::exit(1);
            }

            let program = program.unwrap();

            let mut semantic = SemanticAnalyzer::new();
            semantic.analyze_program(&program);
            
            let mut all_diagnostics = semantic.diagnostics.clone();
            
            let mut linter = meridian_lint::Linter::new();
            linter.lint_program(&program, &semantic);
            
            all_diagnostics.extend(linter.diagnostics);
            
            if !all_diagnostics.is_empty() {
                if cli.ai_diagnostics {
                    let json = serde_json::to_string_pretty(&all_diagnostics).unwrap();
                    println!("{}", json);
                } else {
                    print_diagnostics(&all_diagnostics, false);
                }
                std::process::exit(1);
            } else {
                println!("[]");
            }
        }
        Commands::Test { file, workspace, allow_net, allow_fs, allow_run, allow_all } => {
            let mut visited = HashSet::new();
            let mut diagnostics = Vec::new();
            let mut test_programs = Vec::new();

            if *workspace {
                // Find all .merid files in current directory and subdirectories
                let mut dirs = vec![PathBuf::from(".")];
                while let Some(dir) = dirs.pop() {
                    if let Ok(entries) = fs::read_dir(dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_dir() {
                                if !path.ends_with("target") && !path.ends_with(".git") {
                                    dirs.push(path);
                                }
                            } else if path.extension().is_some_and(|ext| ext == "merid" || ext == "mr") {
                                if let Some(prog) = load_module(&path, &mut visited, &mut diagnostics, None) {
                                    test_programs.push(prog);
                                }
                            }
                        }
                    }
                }
            } else if let Some(f) = file {
                if let Some(prog) = load_module(Path::new(&f), &mut visited, &mut diagnostics, None) {
                    test_programs.push(prog);
                }
            } else {
                eprintln!("Either a file or --workspace must be provided.");
                std::process::exit(1);
            }
            
            if !diagnostics.is_empty() {
                print_diagnostics(&diagnostics, cli.ai_diagnostics);
                std::process::exit(1);
            }

            let mut passed = 0;
            let mut failed = 0;
            let mut all_coverage = HashMap::new();
            let mut total_ir_instructions = 0;

            for mut program in test_programs {
                let mut semantic = SemanticAnalyzer::new();
                if *allow_net { semantic.capabilities.insert("--allow-net".to_string()); }
                if *allow_fs { semantic.capabilities.insert("--allow-fs".to_string()); }
                if *allow_run { semantic.capabilities.insert("--allow-run".to_string()); }
                if *allow_all { semantic.capabilities.insert("--allow-all".to_string()); }
                semantic.analyze_program(&program);
                
                if !semantic.diagnostics.is_empty() {
                    print_diagnostics(&semantic.diagnostics, cli.ai_diagnostics);
                    std::process::exit(1);
                }

                let mut test_functions = Vec::new();
                for stmt in &program.statements {
                    if let Stmt::Function { name, attributes, .. } = stmt {
                        if attributes.contains(&"test".to_string()) {
                            test_functions.push(name.clone());
                        }
                    }
                }

                if test_functions.is_empty() {
                    continue;
                }

                program.statements.extend(semantic.get_monomorphized_statements());
                let struct_layouts = semantic.get_struct_layouts();
                let enum_layouts = semantic.get_enum_layouts();
                let compiler = Compiler::new(semantic.type_map, semantic.resolved_names, semantic.auto_borrows, struct_layouts, enum_layouts);
                let program_ir = compiler.compile(&program);

                for (chunk, _, _) in program_ir.functions.values() {
                    total_ir_instructions += chunk.instructions.len();
                }

                for test_name in test_functions {
                    print!("test {} ... ", test_name);
                    
                    if let Some((chunk, _, _)) = program_ir.functions.get(&test_name) {
                        let mut test_ir = program_ir.clone();
                        test_ir.main_chunk = chunk.clone();
                        
                        let mut vm = VM::new();
                        meridian_stdlib::register_all(&mut vm);
                        let result = vm.run(&test_ir);

                        for (func, ips) in vm.coverage {
                            all_coverage.entry(func).or_insert_with(HashSet::new).extend(ips);
                        }

                        match result {
                            Ok(_) => {
                                println!("ok");
                                passed += 1;
                            }
                            Err(e) => {
                                println!("FAILED");
                                println!("  Runtime Error: {}", e.message);
                                println!("  Stack Trace:");
                                for (func, ip) in e.stack_trace {
                                    println!("    at {} (ip: {})", func, ip);
                                }
                                failed += 1;
                            }
                        }
                    } else {
                        println!("FAILED (not found in IR)");
                        failed += 1;
                    }
                }
            }

            println!("\ntest result: {}. {} passed; {} failed", 
                if failed == 0 { "ok" } else { "FAILED" }, 
                passed, 
                failed
            );

            // Coverage Report
            if total_ir_instructions > 0 {
                let mut hit_instructions = 0;
                for ips in all_coverage.values() {
                    hit_instructions += ips.len();
                }
                let coverage_pct = (hit_instructions as f64 / total_ir_instructions as f64) * 100.0;
                println!("Coverage: {:.2}% ({} / {} instructions hit)", coverage_pct, hit_instructions, total_ir_instructions);
            }

            if failed > 0 {
                std::process::exit(1);
            }
        }

    }
}
