//! # DlinkWM: Dynamic Linking WebAssembly Manager
//! 
//! DlinkWM is a Rust-based WebAssembly (WASM) dynamic calling host that enables
//! loading, calling, and hot-reloading of WASM modules written in multiple languages.
//! 
//! ## Features
//! 
//! - **High Performance**: Built on Wasmtime for excellent WASM execution performance
//! - **Dynamic Loading**: Supports runtime loading and instantiation of WASM modules
//! - **Multi-language Support**: Compatible with WASM modules from Rust, JavaScript, Python, and more
//! - **Hot Reload**: Automatically reloads WASM modules when they change, no host restart needed
//! - **Flexible Calling**: Provides both efficient and flexible calling methods
//! - **Safe & Stable**: Leverages Rust's memory safety guarantees
//! - **Configuration Management**: Supports dynamic configuration with hot reload
//! - **Custom Host Methods**: Allows registering custom host functions that WASM modules can call
//! 
//! ## Quick Start
//! 
//! ```rust
//! use dlink_wm::wasm_manager::{WasmInstanceCache, call_wasm_function};
//! use dlink_wm::config::{DynamicConfig, create_default_config_if_missing};
//! use std::sync::Arc;
//! use anyhow::Result;
//! 
//! fn main() -> Result<()> {
//!     // Initialize configuration
//!     create_default_config_if_missing()?;
//!     let mut dynamic_config = DynamicConfig::new("dlinkwm.toml")?;
//!     dynamic_config.start_watching()?;
//!     
//!     // Create WASM instance cache
//!     let instance_cache = Arc::new(WasmInstanceCache::new());
//!     
//!     // Call WASM function with validation
//!     call_wasm_function(
//!         "wasm/wasm_test.wasm",
//!         "dlinkwm_print_hello_wasm",
//!         &instance_cache,
//!         &dynamic_config
//!     )?;
//!     
//!     Ok(())
//! }
//! ```
//! 
//! ## Core Concepts
//! 
//! - **WasmInstanceCache**: Manages WASM modules and instances, caching them for improved performance
//! - **WasmHotReloader**: Monitors WASM files and automatically reloads them when changes occur
//! - **DynamicConfig**: Loads and monitors configuration files with hot reload support
//! - **Host Methods**: Functions exposed to WASM modules that allow them to interact with the host
//! - **Entry Functions**: Configurable list of allowed functions that can be called from the host
//! 
//! ## Modules
//! 
//! - **wasm_manager**: Core functionality for managing WASM instances and hot reload
//! - **host_import**: Host functions imported by WASM modules
//! - **config**: Configuration management with hot reload
//! - **utils**: Utility functions for WASM memory management and serialization

pub mod host_import;
pub mod utils;
pub mod wasm_manager;
pub mod config;
