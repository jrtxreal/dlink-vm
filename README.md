# DlinkWM - Dynamic Linking WebAssembly Manager

DlinkWM is a Rust-based WASM dynamic calling host that supports loading, calling, and hot-reloading of WASM modules written in multiple languages.

## ✨ Features

- 🚀 **High Performance**: Uses Wasmtime as the WASM runtime for excellent execution performance
- 🔄 **Dynamic Loading**: Supports runtime loading and instantiation of WASM modules
- 🔧 **Multi-language Support**: Compatible with WASM modules compiled from Rust, JavaScript, Python, and other languages
- 🔥 **Hot Reload**: Supports hot updates of WASM modules without restarting the host program
- 📞 **Flexible Calling**: Provides both high-efficiency and flexible calling methods
- 🔒 **Safe and Stable**: Based on Rust's memory safety guarantees, preventing security vulnerabilities in WASM modules

## 📦 Installation

Add the dependency in your Cargo.toml:

```toml
dependencies = {
    dlink-wm = "0.1.0"
}
```

## 🚀 Quick Start

### Example: Loading and Calling a Rust WASM Module

```rust
use dlink_wm::wasm_manager::WasmInstanceCache;
use std::sync::Arc;
use anyhow::Result;

fn main() -> Result<()> {
    // Create WASM instance cache
    let instance_cache = Arc::new(WasmInstanceCache::new());
    
    // Load and instantiate WASM module
    let instance_store = instance_cache.load_and_instantiate("wasm_bin/rust_wasm.wasm")?;
    
    // Get instance and store
    let mut guard = instance_store.write().unwrap();
    let (ref mut instance, ref mut store) = *guard;
    
    // Call WASM export function
    let extern_val = instance.get_export(&mut *store, "dlinkwm_rust_wasm_test")
        .ok_or_else(|| anyhow!("Failed to find export"))?;
    
    let func = extern_val.into_func()
        .ok_or_else(|| anyhow!("Export is not a function"))?;
    
    let test_func = func.typed::<(), ()>(&mut *store)?;
    test_func.call(&mut *store, ())?;
    
    Ok(())
}
```

### Example: Enabling Hot Reload

```rust
use dlink_wm::wasm_manager::{WasmInstanceCache, WasmHotReloader};
use std::sync::Arc;
use anyhow::Result;

fn main() -> Result<()> {
    // Create WASM instance cache
    let instance_cache = Arc::new(WasmInstanceCache::new());
    
    // Load and instantiate WASM module
    let wasm_path = "wasm_bin/rust_wasm.wasm";
    let instance_store = instance_cache.load_and_instantiate(wasm_path)?;
    
    // Extract WASM file directory
    let wasm_dir = std::path::Path::new(wasm_path)
        .parent()
        .ok_or_else(|| anyhow!("Failed to get WASM directory"))?
        .to_str()
        .ok_or_else(|| anyhow!("Failed to convert directory path"))?;
    
    // Start hot reload manager
    let hot_reloader = WasmHotReloader::new(instance_cache.clone(), wasm_dir);
    hot_reloader.start();
    
    // Keep program running
    println!("Hot reload enabled. Press Ctrl+C to exit.");
    std::thread::park();
    
    Ok(())
}
```

## 📚 API Documentation

### WasmInstanceCache

WASM instance cache for managing WASM module loading and instantiation.

```rust
use dlink_wm::wasm_manager::WasmInstanceCache;

// Create instance cache
let cache = WasmInstanceCache::new();

// Load and instantiate WASM module
let instance_store = cache.load_and_instantiate("path/to/module.wasm")?;

// Clear cache
cache.clear_cache("path/to/module.wasm");

// Hot reload module
let new_instance_store = cache.hot_reload("path/to/module.wasm")?;
```

### WasmHotReloader

WASM hot reload manager for monitoring WASM file changes and automatically reloading them.

```rust
use dlink_wm::wasm_manager::WasmHotReloader;

// Create hot reload manager
let hot_reloader = WasmHotReloader::new(instance_cache.clone(), "wasm_dir");

// Start hot reload
hot_reloader.start();
```

## 📝 Project Structure

```
dlink-wm/
├── src/
│   ├── config.rs         # Configuration
│   ├── host_import.rs     # Host import functions
│   ├── lib.rs            # Library entry point
│   ├── utils.rs          # Utility functions
│   └── wasm_manager.rs   # WASM instance management
├── examples/
│   ├── rust_call_wasm.rs    # Rust calling WASM example
│   └── wasm_calll_rust.rs   # WASM calling Rust example
├── Cargo.toml          # Project configuration
├── Cargo.lock          # Dependency lock file
├── LICENSE             # MIT License
├── README.md           # Project documentation
└── dlinkwm.toml        # Project configuration
```

## 🛠️ Host Import Functions

### High Efficiency Type
- `host_u32_calc(a: u32, b: u32, op: u32) -> u32`: 32-bit unsigned integer general calculation
- `host_bytes_echo(in_ptr: i32, in_len: i32, out_ptr: i32) -> i32`: Byte array echo

### Flexible Type
- `host_json_execute(req_ptr: i32, req_len: i32, resp_ptr: i32) -> i32`: General JSON request-response
- `host_json_store(data_ptr: i32, data_len: i32) -> u64`: General JSON data storage

## 📄 License

DlinkWM is licensed under the MIT License.

## 🤝 Contributing

Issues and Pull Requests are welcome!

## 📞 Contact

- Project URL: https://github.com/dlinkwm/dlink-wm
- Issue Tracker: https://github.com/dlinkwm/dlink-wm/issues

---

**DlinkWM - Making WASM calling simpler and more efficient!**
