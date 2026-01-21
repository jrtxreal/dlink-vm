//! DlinkWM Custom Host Methods Example
//! Demonstrates how to register custom host methods dynamically in application code

use dlink_wm::host_import::{register_host_method, SerializationFormat};
use dlink_wm::wasm_manager::{WasmInstanceCache, WasmHotReloader, call_wasm_function};
use dlink_wm::config::{DynamicConfig, create_default_config_if_missing, get_default_config_path};
use std::sync::Arc;
use clap::Parser;
use env_logger::Env;
use anyhow::{anyhow, Result as AnyResult, Result};
use std::io::{BufRead, Write};
use serde::{Serialize, Deserialize};

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

/// JSON params wrapper (replicated from the host_import module for example purposes)
#[derive(Debug, Serialize, Deserialize)]
struct JsonParams<T> {
    data: T,
}

// -------------------------- Custom Host Method Handlers --------------------------

/// Custom greeting method handler - 返回字符串给WASM
fn custom_greet_handler(params_bytes: Vec<u8>, format: SerializationFormat) -> AnyResult<(bool, Vec<u8>)> {
    match format {
        SerializationFormat::Json => {
            #[derive(Debug, Serialize, Deserialize)]
            struct GreetParams { name: String }
            
            let params: JsonParams<GreetParams> = serde_json::from_slice(&params_bytes)?;
            let result = format!("Hello from custom handler, {}!", params.data.name);
            Ok((true, serde_json::to_vec(&result)?))
        },
        _ => Err(anyhow!("Format not supported for custom_greet method")),
    }
}

fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Parse command line arguments
    let args = Args::parse();

    println!("=== DlinkWM Custom Host Methods Example ===");
    println!("WASM file path: {}", args.wasm_path);
    println!("Enable hot reload: {}", args.hot_reload);
    println!("Usage:");
    println!("  Enter 'call' to invoke WASM methods with the latest WASM file");
    println!("  Press Ctrl+C to exit the program");
    println!();

    // -------------------------- Register Custom Host Methods --------------------------
    println!("🔧 Registering custom host methods...");
    
    // Register only the custom greeting method
    if register_host_method("custom_greet", custom_greet_handler) {
        println!("✅ Successfully registered 'custom_greet' method");
    } else {
        println!("⚠️  Failed to register 'custom_greet' method (already exists)");
    }
    
    println!();

    // -------------------------- Load WASM and Use Methods --------------------------
    
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

    // If hot reload is enabled, start hot reload monitoring
    if args.hot_reload {
        println!("� Enabling hot reload functionality");
        
        // Extract WASM file directory
        let wasm_dir = std::path::Path::new(&args.wasm_path)
            .parent()
            .ok_or_else(|| anyhow!("Failed to get WASM directory"))?
            .to_str()
            .ok_or_else(|| anyhow!("Failed to convert directory path to string"))?;
        
        // Start hot reload manager
        let hot_reloader = WasmHotReloader::new(instance_cache.clone(), wasm_dir);
        hot_reloader.start();
        
        println!("✅ Hot reload monitoring started");
        println!("💡 Tip: Changes to WASM module will take effect automatically after rebuilding");
        println!("   Changes to configuration file will also be detected");
        println!();
    }

    println!("🎉 Custom host methods registered and available for WASM to use");
    println!("💡 Available host methods:");
    println!("   - custom_greet: Greets a user and returns a string");
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
            
            println!("� Trying entry functions from config: {:?}", entry_functions);
            println!("📞 Attempting to call WASM entry functions...");
            
            // Call each configured entry function using call_wasm_function
                match call_wasm_function(&args.wasm_path, "dlinkwm_call_host_method", &instance_cache, &dynamic_config) {
                    Ok(_) => println!("✅ Successfully called dlinkwm_call_host_method"),
                    Err(e) => {
                        println!("❌ Error calling function dlinkwm_call_host_method: {}", e);
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