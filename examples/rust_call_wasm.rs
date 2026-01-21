//! DlinkWM Basic Usage Example
//! Demonstrates how to load, call, and hot reload WASM modules with dynamic configuration

use dlink_wm::wasm_manager::{WasmInstanceCache, call_wasm_function};
use dlink_wm::config::{DynamicConfig, create_default_config_if_missing, get_default_config_path};
use std::sync::Arc;
use clap::Parser;
use env_logger::Env;
use anyhow::Result;
use std::io::{BufRead, Write};

/// Example command line arguments
#[derive(Parser, Debug)]
struct Args {
    /// WASM file path
    #[arg(short, long, default_value = "wasm/wasm_test.wasm")]
    wasm_path: String,
    /// Enable hot reload
    #[arg(short = 'r', long, default_value_t = true)]
    hot_reload: bool,
    /// Configuration file path
    #[arg(short, long)]
    config_path: Option<String>,
}

fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Parse command line arguments
    let args = Args::parse();

    println!("=== DlinkWM Interactive WASM Call Example ===");
    println!("WASM file path: {}", args.wasm_path);
    println!("Usage:");
    println!("  Enter 'call' to invoke WASM methods with the latest WASM file");
    println!("  Press Ctrl+C to exit the program");
    println!();

    // Get config path
    let config_path = args.config_path.unwrap_or_else(get_default_config_path);
    println!("Using configuration file: {}", config_path);
    
    // Create default config if missing
    create_default_config_if_missing()?;
    println!("✅ Default configuration created if missing");
    
    // Create dynamic config with hot reload support
    let mut dynamic_config = DynamicConfig::new(&config_path)?;
    println!("✅ Dynamic configuration created");
    
    // Start watching config file for changes
    dynamic_config.start_watching()?;
    println!("✅ Configuration watcher started");
    println!();

    // Create WASM instance cache
    let instance_cache = Arc::new(WasmInstanceCache::new());
    println!("✅ WASM instance cache created");
    println!();

    // Create a reader for user input
    let mut reader = std::io::stdin().lock();
    
    loop {
        print!("Enter command: ");
        std::io::stdout().flush()?;
        
        let mut input = String::new();
        reader.read_line(&mut input)?;
        let input = input.trim();
        
        if input == "call" {
            println!("\n📞 Calling WASM methods with latest WASM file...");
            
            // Get entry functions for this specific WASM file
            let entry_functions: Vec<String> = dynamic_config.get_entry_functions_for_file(&args.wasm_path);
            
            // If no entry functions are defined for this file, print message and skip calling
            if entry_functions.is_empty() {
                println!("⚠️  No entry functions configured for this file in config");
                println!("   Please check the dlinkwm.toml file to configure entry functions for: {}", args.wasm_path);
                println!();
                continue;
            }
            
            println!("🔍 Trying entry functions from config: {:?}", entry_functions);
            println!("📞 Attempting to call WASM entry functions...");
            
            // Call each configured entry function using call_wasm_function
                match call_wasm_function(&args.wasm_path, "dlinkwm_print_hello_wasm", &instance_cache, &dynamic_config) {
                    Ok(_) => println!(),
                    Err(e) => {
                        println!("❌ Error calling function dlinkwm_print_hello_wasm: {}", e);
                        println!();
                    }
                }
            
            println!();
        } else if !input.is_empty() {
            println!("\n⚠️  Unknown command: {}", input);
            println!("Please enter 'call' to invoke WASM methods or Ctrl+C to exit");
            println!();
        }
    }
}