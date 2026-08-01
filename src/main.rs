#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
use clap::Parser;
use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

use il2dump_lib::{
    binary, binary_reader, config, decompiler, il2cpp_binary_structures, il2cpp_executor, metadata,
};

#[derive(Parser, Debug)]
#[command(
    name = "il2dump",
    author = "Mathias Bynens",
    version = "0.1.0",
    about = "Portable il2dump in Rust"
)]
struct Args {
    /// Path to the il2cpp executable binary (PE, ELF, Mach-O).
    #[arg(index = 1)]
    executable: PathBuf,

    /// Path to the global-metadata.dat file.
    #[arg(index = 2)]
    metadata: PathBuf,

    /// Directory where the output files will be written.
    #[arg(index = 3)]
    output_dir: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Read the config.json file.
    let config = if Path::new("config.json").exists() {
        println!("Loading config.json...");
        let config_str = fs::read_to_string("config.json")?;
        serde_json::from_str::<config::Config>(&config_str).unwrap_or_else(|e| {
            println!(
                "Warning: failed to parse config.json ({}), using defaults",
                e
            );
            config::Config::default()
        })
    } else {
        println!("config.json not found, using default configuration.");
        config::Config::default()
    };

    // Load global-metadata.dat.
    println!("Loading metadata: {:?}", args.metadata);
    let metadata_bytes = fs::read(&args.metadata)?;
    let metadata = metadata::Metadata::load(metadata_bytes)?;
    println!("Metadata version: {}", metadata.version);

    // Load the executable binary.
    println!("Loading executable: {:?}", args.executable);
    let exec_bytes = fs::read(&args.executable)?;
    let mut binary_file = binary::BinaryFile::parse(exec_bytes)?;
    println!(
        "Executable format parsed: 32bit={}, endian={:?}, base=0x{:X}",
        binary_file.is_32bit, binary_file.endian, binary_file.image_base
    );

    // Determine the target version to use for binary parsing.
    let version = if config.force_il2cpp_version {
        config.force_version
    } else {
        metadata.version
    };
    println!("Target IL2CPP version: {}", version);

    // Search for the CodeRegistration and MetadataRegistration addresses.
    println!("Searching for registration structures...");
    let (mut code_reg, mut metadata_reg) = binary_file.symbol_search();
    let mut found = code_reg > 0 && metadata_reg > 0;

    if found {
        println!("Detected via Symbols!");
    } else {
        // Try to perform a pattern search.
        println!("Symbols not found. Trying heuristic search...");
        let method_count = metadata
            .method_defs
            .iter()
            .filter(|x| x.method_index >= 0)
            .count();
        let (cr, mr) = binary_file.plus_search(
            version,
            method_count,
            metadata.type_defs.len(),
            metadata.metadata_usages_count,
            metadata.image_defs.len(),
        );
        if cr > 0 && mr > 0 {
            code_reg = cr;
            metadata_reg = mr;
            found = true;
            println!("Detected via Heuristic Search!");
        }
    }

    if !found {
        println!("Heuristic search failed. Please provide registration addresses manually.");
        let stdin = io::stdin();
        let mut handle = stdin.lock();

        print!("Input CodeRegistration (hex): ");
        io::Write::flush(&mut io::stdout())?;
        let mut cr_str = String::new();
        handle.read_line(&mut cr_str)?;
        code_reg = u64::from_str_radix(cr_str.trim().trim_start_matches("0x"), 16)?;

        print!("Input MetadataRegistration (hex): ");
        io::Write::flush(&mut io::stdout())?;
        let mut mr_str = String::new();
        handle.read_line(&mut mr_str)?;
        metadata_reg = u64::from_str_radix(mr_str.trim().trim_start_matches("0x"), 16)?;
    }

    // Initialize the executor.
    let executor =
        il2cpp_executor::Il2CppExecutor::new(metadata, binary_file, code_reg, metadata_reg)?;

    println!(
        "CodeRegistration: 0x{:X}",
        executor.code_registration_address
    );
    println!(
        "MetadataRegistration: 0x{:X}",
        executor.metadata_registration_address
    );

    // Trigger the decompiler to generate the output files.
    let output_dir = args.output_dir.unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&output_dir)?;

    println!("Decompiling metadata definitions to dump.cs and script.json...");
    let decompiler = decompiler::Decompiler::new(&executor);
    decompiler.decompile(&config, &output_dir)?;

    println!("Dump completed successfully!");
    Ok(())
}
