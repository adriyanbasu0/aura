use std::process;
// hi
fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Aura Compiler v0.1.0");
        eprintln!("Usage: aura <command> [options]");
        eprintln!("Commands:");
        eprintln!("  build [options] <source.aura>  Compile source to .aura binary");
        eprintln!("  run <binary.aura>              Build and run");
        eprintln!("  check <source.aura>            Type check only");
        eprintln!("  dump <binary.aura>             Dump binary info");
        eprintln!("Options:");
        eprintln!("  -o <output.aura>  Specify output file");
        process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "build" => {
            let mut source = None;
            let mut output = None;
            let mut i = 2;
            while i < args.len() {
                if args[i] == "-o" && i + 1 < args.len() {
                    output = Some(args[i + 1].clone());
                    i += 2;
                } else if args[i].starts_with('-') {
                    eprintln!("Unknown option: {}", args[i]);
                    process::exit(1);
                } else {
                    source = Some(args[i].clone());
                    i += 1;
                }
            }

            let source = match source {
                Some(s) => s,
                None => {
                    eprintln!("Usage: aura build [-o <output.aura>] <source.aura>");
                    process::exit(1);
                }
            };

            let result = aura_compiler::compile_file(&source, output.as_deref());
            if let Err(e) = result {
                eprintln!("Error: {:?}", e);
                process::exit(1);
            }
        }
        "check" => {
            if args.len() < 3 {
                eprintln!("Usage: aura check <source.aura>");
                process::exit(1);
            }
            let result = aura_compiler::typecheck_file(&args[2]);
            if let Err(e) = result {
                eprintln!("Error: {:?}", e);
                process::exit(1);
            }
            println!("Type check passed");
        }
        "dump" => {
            if args.len() < 3 {
                eprintln!("Usage: aura dump <binary.aura>");
                process::exit(1);
            }
            if let Err(e) = aura_compiler::dump_binary(&args[2]) {
                eprintln!("Error: {:?}", e);
                process::exit(1);
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            process::exit(1);
        }
    }
}
