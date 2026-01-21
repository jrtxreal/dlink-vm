# DlinkWM: Dynamic Linking WebAssembly Manager

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
DlinkWM is a Rust-based WebAssembly (WASM) dynamic calling host that enables loading, calling, and hot-reloading of WASM modules written in multiple languages. It provides a robust and efficient way to integrate WASM components into your Rust applications with minimal friction.

## 🚀 Features

- **High Performance**: Built on Wasmtime for excellent WASM execution performance
- **Dynamic Loading**: Supports runtime loading and instantiation of WASM modules
- **Multi-language Support**: Compatible with WASM modules from Rust, JavaScript, Python, and more
- **Hot Reload**: Automatically reloads WASM modules when they change, no host restart needed
- **Flexible Calling**: Provides both efficient and flexible calling methods
- **Safe & Stable**: Leverages Rust's memory safety guarantees
- **Configuration Management**: Supports dynamic configuration with hot reload
- **Custom Host Methods**: Allows registering custom host functions that WASM modules can call

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
dependencies = {
    "dlink-wm" = "0.1.0"
}
```

Or install from source:

```bash
git clone https://github.com/dlinkwm/dlink-wm.git
cd dlink-wm
cargo build --release
```

## 📖 Quick Start

### Basic Usage

```rust
use dlink_wm::wasm_manager::{WasmInstanceCache, call_wasm_function};
use dlink_wm::config::{DynamicConfig, create_default_config_if_missing};
use std::sync::Arc;
use anyhow::Result;

fn main() -> Result<()> {
    // Initialize configuration
    create_default_config_if_missing()?;
    let mut dynamic_config = DynamicConfig::new("dlinkwm.toml")?;
    dynamic_config.start_watching()?;
    
    // Create WASM instance cache
    let instance_cache = Arc::new(WasmInstanceCache::new());
    
    // Call WASM function with validation
    call_wasm_function(
        "wasm/wasm_test.wasm",
        "dlinkwm_print_hello_wasm",
        &instance_cache,
        &dynamic_config
    )?;
    
    Ok(())
}
```

### Hot Reload Example

```rust
use dlink_wm::wasm_manager::{WasmInstanceCache, WasmHotReloader};
use dlink_wm::config::{DynamicConfig, create_default_config_if_missing};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use anyhow::Result;

fn main() -> Result<()> {
    // Initialize configuration
    create_default_config_if_missing()?;
    let dynamic_config = DynamicConfig::new("dlinkwm.toml")?;
    
    // Create WASM instance cache
    let instance_cache = Arc::new(WasmInstanceCache::new());
    
    // Start hot reloader
    let reloader = WasmHotReloader::new(instance_cache.clone(), dynamic_config.clone());
    reloader.start_watching()?;
    
    // Keep the application running to observe hot reload
    loop {
        println!("Host is running... WASM modules will be hot-reloaded when changed");
        thread::sleep(Duration::from_secs(5));
    }
}
```

## 🛠️ Configuration

DlinkWM uses a TOML configuration file (`dlinkwm.toml`) to manage entry functions for different WASM modules. Here's an example configuration:

```toml
# DlinkWM Configuration File
# This file can be modified to dynamically change entry functions without restarting

# Entry Functions Configuration
# Define specific entry functions for different WASM files
# Key: WASM file path (relative or absolute)
# Value: List of entry functions to try for this specific file
# The functions are tried in the order they appear in the list

[entry_functions]
# Example configuration for wasm_test.wasm
# Tries to call dlinkwm_print_hello_wasm first, then dlinkwm_test_host_methods if the first fails
"wasm/wasm_test.wasm" = [
"dlinkwm_print_hello_wasm",    # First try this function
"dlinkwm_call_host_method"      # Then try this function if the first fails
]

# Example configuration for hello_simple.wasm
# Only tries a single entry function
"wasm/hello_simple.wasm" = ["dlinkwm_simple_entry"]

# Example configuration for hello_only.wasm
# Only tries a single entry function
"wasm/hello_only.wasm" = ["dlinkwm_hello_entry"]

# Example configuration for a custom WASM file
# You can add more entries like this for your own WASM files
# "path/to/your/wasm/file.wasm" = ["your_entry_function1", "your_entry_function2"]
```

## 📁 Project Structure

```
dlink-wm/
├── examples/            # Example code
│   ├── rust_call_wasm.rs    # Example of calling WASM from Rust
│   └── wasm_calll_rust.rs   # Example of calling Rust from WASM
├── src/                 # Source code
│   ├── config.rs        # Configuration management
│   ├── host_import.rs   # Host functions imported by WASM modules
│   ├── lib.rs           # Main library file
│   ├── utils.rs         # Utility functions
│   └── wasm_manager.rs  # WASM module management
├── wasm/                # WASM files
│   └── wasm_test.wasm   # Test WASM module
├── wasm_test/           # WASM test module source
│   ├── src/
│   ├── Cargo.lock
│   └── Cargo.toml
├── .gitignore
├── Cargo.lock
├── Cargo.toml
├── LICENSE
└── dlinkwm.toml         # Configuration file
```

## 🔧 Core Concepts

### WasmInstanceCache
Manages WASM modules and instances, caching them for improved performance. It handles the loading, instantiation, and management of WASM modules.

### WasmHotReloader
Monitors WASM files and automatically reloads them when changes occur, enabling hot reload functionality without host restarts.

### DynamicConfig
Loads and monitors configuration files with hot reload support, allowing configuration changes to take effect without application restarts.

### Host Methods
Functions exposed to WASM modules that allow them to interact with the host environment. These can be custom functions registered by the host application.

### Entry Functions
Configurable list of allowed functions that can be called from the host. Defined in the configuration file for each WASM module.

## 🎯 Use Cases

- **Plugin Systems**: Create extensible applications with WASM-based plugins
- **Microservices**: Run lightweight microservices in WASM sandboxes
- **Game Modding**: Allow safe, sandboxed game modifications
- **Runtime Extensions**: Extend applications at runtime without restarts
- **Cross-language Integration**: Integrate code from multiple languages seamlessly
- **Hot-reloadable Business Logic**: Update business logic without downtime

## 🔍 Examples

### Basic Examples

#### Calling Rust from WASM

```rust
// In your WASM module
#[no_mangle]
pub extern "C" fn dlinkwm_print_hello_wasm() {
    println!("Hello from WASM!");
    // Call host method
    unsafe {
        dlinkwm_host_print(b"Calling host method from WASM!\0");
    }
}

// Import host function
#[link(wasm_import_module = "host")]
extern "C" {
    fn dlinkwm_host_print(ptr: *const u8);
}
```

#### Calling WASM from Rust

```rust
// In your Rust host application
use dlink_wm::wasm_manager::{WasmInstanceCache, call_wasm_function};
use dlink_wm::config::{DynamicConfig, create_default_config_if_missing};
use std::sync::Arc;
use anyhow::Result;

fn main() -> Result<()> {
    // Initialize configuration
    create_default_config_if_missing()?;
    let mut dynamic_config = DynamicConfig::new("dlinkwm.toml")?;
    dynamic_config.start_watching()?;
    
    // Create WASM instance cache
    let instance_cache = Arc::new(WasmInstanceCache::new());
    
    // Call WASM function
    call_wasm_function(
        "wasm/wasm_test.wasm",
        "dlinkwm_print_hello_wasm",
        &instance_cache,
        &dynamic_config
    )?;
    
    Ok(())
}
```

### Complete Examples

#### rust_call_wasm.rs - Interactive WASM Call Example

This example demonstrates how to load, call, and hot reload WASM modules with dynamic configuration in an interactive way.

**Features:**
- Command-line argument parsing
- Interactive user input
- Dynamic configuration with hot reload
- WASM instance caching
- Error handling and logging

**Usage:**
```bash
cargo run --example rust_call_wasm -- --wasm-path wasm/wasm_test.wasm
```

**Key functionalities:**
- Creates default configuration if missing
- Starts configuration file watching
- Creates WASM instance cache for efficient module management
- Provides interactive prompt to call WASM methods
- Automatically uses the latest WASM file when calling

**Example output:**
```
=== DlinkWM Interactive WASM Call Example ===
WASM file path: wasm/wasm_test.wasm
Usage:
  Enter 'call' to invoke WASM methods with the latest WASM file
  Press Ctrl+C to exit the program

Using configuration file: dlinkwm.toml
✅ Default configuration created if missing
✅ Dynamic configuration created
✅ Configuration watcher started

✅ WASM instance cache created

Enter command: call

📞 Calling WASM methods with latest WASM file...
🔍 Trying entry functions from config: ["dlinkwm_print_hello_wasm", "dlinkwm_call_host_method"]
📞 Attempting to call WASM entry functions...
Hello from WASM!
Calling host method from WASM!

✅ Successfully called dlinkwm_print_hello_wasm

Enter command:
```

#### wasm_call_rust.rs - Custom Host Methods Example

This example demonstrates how to register custom host methods dynamically in application code and how WASM modules can call these host methods.

**Features:**
- Custom host method registration
- Serialization/deserialization of method parameters
- Hot reload functionality
- Command-line argument parsing
- Interactive user input

**Usage:**
```bash
cargo run --example wasm_call_rust -- --wasm-path wasm/wasm_test.wasm
```

**Key functionalities:**
- Registers custom host methods (e.g., `custom_greet`)
- Handles different serialization formats
- Starts hot reload monitoring for WASM files
- Provides interactive prompt to call WASM methods
- Demonstrates bidirectional communication between host and WASM

**Example custom host method:**
```rust
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
```

**Example output:**
```
=== DlinkWM Custom Host Methods Example ===
WASM file path: wasm/wasm_test.wasm
Enable hot reload: true
Usage:
  Enter 'call' to invoke WASM methods with the latest WASM file
  Press Ctrl+C to exit the program

🔧 Registering custom host methods...
✅ Successfully registered 'custom_greet' method

Using configuration file: dlinkwm.toml
✅ Default configuration created if missing
✅ Dynamic configuration created
✅ Configuration watcher started

✅ WASM instance cache created
✅ Hot reload monitoring started
💡 Tip: Changes to WASM module will take effect automatically after rebuilding
   Changes to configuration file will also be detected

🎉 Custom host methods registered and available for WASM to use
💡 Available host methods:
   - custom_greet: Greets a user and returns a string

Enter command: call

📞 Calling WASM methods with latest WASM file...
🔍 Trying entry functions from config: ["dlinkwm_print_hello_wasm", "dlinkwm_call_host_method"]
📞 Attempting to call WASM entry functions...
Hello from WASM!
Calling host method from WASM!
Calling custom host method...
Result from custom host method: Hello from custom handler, WASM!

✅ Successfully called dlinkwm_call_host_method

Enter command:
```

## 📚 API Reference

### WasmInstanceCache

```rust
pub struct WasmInstanceCache {
    // Internal implementation
}

impl WasmInstanceCache {
    pub fn new() -> Self
    pub fn get_or_load(&self, wasm_path: &str, config: &DynamicConfig) -> Result<Instance>
    pub fn reload(&self, wasm_path: &str, config: &DynamicConfig) -> Result<Instance>
}
```

### DynamicConfig

```rust
pub struct DynamicConfig {
    // Internal implementation
}

impl DynamicConfig {
    pub fn new(config_path: &str) -> Result<Self>
    pub fn start_watching(&mut self) -> Result<()>
    pub fn get_entry_functions(&self, wasm_path: &str) -> Option<&Vec<String>>
}
```

### HostImport

```rust
pub fn register_host_functions(linker: &mut Linker<WasmtimeStoreData>) -> Result<()>
// Add custom host functions here
```

## 🔧 Development

### Building the Project

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Building WASM Modules

```bash
cd wasm_test
cargo build --target wasm32-wasi
cp target/wasm32-wasi/debug/wasm_test.wasm ../wasm/
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Guidelines

1. Follow the existing code style
2. Add tests for new features
3. Update documentation as needed
4. Submit pull requests to the `main` branch

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 📞 Support

If you encounter any issues or have questions, please open an issue on GitHub.

## 🙏 Acknowledgments

- [Wasmtime](https://github.com/bytecodealliance/wasmtime) - For the excellent WASM runtime
- [Rust](https://www.rust-lang.org/) - For the safe and performant language
- [Serde](https://github.com/serde-rs/serde) - For serialization/deserialization
- [Notify](https://github.com/notify-rs/notify) - For file system notifications

## 📈 Roadmap

- [ ] Add support for more WASM runtimes
- [ ] Implement async support
- [ ] Add more host-side utilities
- [ ] Improve error handling and reporting
- [ ] Add comprehensive documentation and examples
- [ ] Implement WASM module sandboxing
- [ ] Add support for WASI preview2

---

Made with ❤️ by the DlinkWM Team
