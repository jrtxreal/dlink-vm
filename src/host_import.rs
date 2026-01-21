//! # Host Import Functions
//! 
//! This module defines the host functions that are exposed to WASM modules.
//! It provides a universal invocation interface that allows WASM modules to call
//! custom host methods dynamically, along with memory management functions.

use wasmtime::{Caller, Store, Linker, Engine};
use wasmtime_wasi::WasiCtx;
use wasmtime_wasi::WasiCtxBuilder;
use crate::utils::{read_wasm_memory, write_wasm_memory};
use std::sync::{Arc, RwLock, LazyLock};
use anyhow::{Result as AnyResult, Result};
use std::collections::HashMap;

// -------------------------- Universal Invocation Interface --------------------------

/// # Method Handler Type
/// 
/// Type alias for host method handlers. These functions receive serialized parameters
/// and return a serialized response along with a success status.
/// 
/// # Parameters
/// 
/// - `Vec<u8>`: Serialized parameters in the specified format
/// - `SerializationFormat`: Format used for serialization
/// 
/// # Returns
/// 
/// A tuple containing:
/// - `bool`: Success status (true for success, false for error)
/// - `Vec<u8>`: Serialized response bytes
pub type MethodHandler = fn(Vec<u8>, SerializationFormat) -> AnyResult<(bool, Vec<u8>)>;

/// # Serialization Format
/// 
/// Enum representing the supported serialization formats for communication
/// between WASM modules and the host.
#[derive(Debug, Clone, Copy)]
pub enum SerializationFormat {
    /// JSON serialization format
    Json,
    /// Bincode serialization format
    Bincode,
    /// Protocol Buffers serialization format
    Protobuf,
    /// FlatBuffers serialization format
    FlatBuffers,
}

/// # Host Method Registry
/// 
/// Global registry that stores all host functions available to WASM modules.
/// This registry is thread-safe and can be modified at runtime.
static HOST_METHOD_REGISTRY: LazyLock<Arc<RwLock<HashMap<String, MethodHandler>>>> = LazyLock::new(|| {
    Arc::new(RwLock::new(HashMap::new()))
});

/// # Register a Host Method Dynamically
/// 
/// Registers a new host method that can be called by WASM modules using the
/// universal invocation interface.
/// 
/// # Parameters
/// 
/// - `method_name`: Name of the method to register. This is the name WASM modules
///   will use to call this method.
/// - `handler`: Function pointer to the handler that will be called when the
///   method is invoked from WASM.
/// 
/// # Returns
/// 
/// `true` if the method was registered successfully, `false` if the method name
/// is already registered.
/// 
/// # Example
/// 
/// ```rust
/// use dlink_wm::host_import::{register_host_method, SerializationFormat};
/// use anyhow::{anyhow, Result as AnyResult};
/// 
/// fn custom_greet_handler(params: Vec<u8>, format: SerializationFormat) -> AnyResult<(bool, Vec<u8>)> {
///     match format {
///         SerializationFormat::Json => {
///             // Handle JSON parameters...
///             Ok((true, b"{\"greeting\":\"Hello from host\"}".to_vec()))
///         },
///         _ => Err(anyhow!("Unsupported format")),
///     }
/// }
/// 
/// // Register the method
/// register_host_method("custom_greet", custom_greet_handler);
/// ```
pub fn register_host_method(method_name: &str, handler: MethodHandler) -> bool {
    let mut registry = HOST_METHOD_REGISTRY.write().unwrap();
    registry.insert(method_name.to_string(), handler).is_none()
}

/// # Unregister a Host Method
/// 
/// Removes a previously registered host method from the registry.
/// 
/// # Parameters
/// 
/// - `method_name`: Name of the method to unregister.
/// 
/// # Returns
/// 
/// `true` if the method was unregistered successfully, `false` if the method
/// was not found in the registry.
pub fn unregister_host_method(method_name: &str) -> bool {
    let mut registry = HOST_METHOD_REGISTRY.write().unwrap();
    registry.remove(method_name).is_some()
}

/// # Check if a Host Method Exists
/// 
/// Verifies if a host method with the given name is registered.
/// 
/// # Parameters
/// 
/// - `method_name`: Name of the method to check.
/// 
/// # Returns
/// 
/// `true` if the method exists in the registry, `false` otherwise.
pub fn has_host_method(method_name: &str) -> bool {
    let registry = HOST_METHOD_REGISTRY.read().unwrap();
    registry.contains_key(method_name)
}

/// # Universal Invocation Function
/// 
/// Universal interface for WASM modules to call host methods. All host method
/// calls from WASM go through this function.
/// 
/// # Parameters
/// 
/// - `caller`: WASM caller context
/// - `method_name_ptr`: Pointer to the method name in WASM memory
/// - `method_name_len`: Length of the method name in bytes
/// - `format_type`: Serialization format identifier (0=JSON, 1=Bincode, 2=Protobuf, 3=FlatBuffers)
/// - `params_ptr`: Pointer to the serialized parameters in WASM memory
/// - `params_len`: Length of the serialized parameters in bytes
/// - `ret_ptr`: Pointer to write the serialized response to in WASM memory
/// 
/// # Returns
/// 
/// Status code:
/// - `0`: Success
/// - `1`: Method not found
/// - `2`: Format error
/// - `3`: Execution error
/// 
/// # Response Format
/// 
/// The response is written to the memory location specified by `ret_ptr` in the following format:
/// - `0-3 bytes`: Status code (0 for success, 1 for failure)
/// - `4-7 bytes`: Response data length
/// - `8+ bytes`: Response data
#[export_name = "universal_invoke"]
pub fn universal_invoke(
    mut caller: Caller<'_, WasiCtx>,
    method_name_ptr: i32,
    method_name_len: i32,
    format_type: i32,
    params_ptr: i32,
    params_len: i32,
    ret_ptr: i32,
) -> i32 {
    // Get WASM memory instance
    let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
        Some(mem) => mem,
        None => return 1, // Memory not found
    };

    // Read method name from WASM memory
    let method_name_bytes = match read_wasm_memory(&memory, &caller, method_name_ptr, method_name_len) {
        Ok(bytes) => bytes,
        Err(_) => return 1, // Failed to read method name
    };
    let method_name = match String::from_utf8(method_name_bytes) {
        Ok(name) => name,
        Err(_) => return 1, // Invalid UTF-8 encoding
    };

    // Determine serialization format from format type
    let format = match format_type {
        0 => SerializationFormat::Json,
        1 => SerializationFormat::Bincode,
        2 => SerializationFormat::Protobuf,
        3 => SerializationFormat::FlatBuffers,
        _ => return 2, // Invalid format type
    };

    // Read serialized parameters from WASM memory
    let params_bytes = match read_wasm_memory(&memory, &caller, params_ptr, params_len) {
        Ok(bytes) => bytes,
        Err(_) => return 2, // Failed to read parameters
    };

    // Find and call the registered handler
    match HOST_METHOD_REGISTRY.read().unwrap().get(method_name.as_str()) {
        Some(handler) => {
            match handler(params_bytes, format) {
                Ok((success, ret_bytes)) => {
                    // Write status code (4 bytes, little-endian)
                    let status: u32 = if success { 1 } else { 0 };
                    let status_bytes = status.to_le_bytes();
                    if write_wasm_memory(&memory, &mut caller, ret_ptr, &status_bytes).is_err() {
                        return 3;
                    }
                    
                    // Write response length (4 bytes, little-endian)
                    let len_bytes = (ret_bytes.len() as u32).to_le_bytes();
                    if write_wasm_memory(&memory, &mut caller, ret_ptr + 4, &len_bytes).is_err() {
                        return 3;
                    }
                    
                    // Write response data
                    if write_wasm_memory(&memory, &mut caller, ret_ptr + 8, &ret_bytes).is_err() {
                        return 3;
                    }
                    
                    0 // Success
                },
                Err(_) => 3, // Execution error
            }
        },
        None => 1, // Method not found
    }
}

// -------------------------- Store and Linker Configuration --------------------------

/// # Initialize Store and WASI Context
/// 
/// Creates a new WASM store with a WASI context configured to inherit stdio.
/// 
/// # Returns
/// 
/// A tuple containing:
/// - `Store<WasiCtx>`: The WASM store instance
/// - `WasiCtx`: The WASI context
/// - `Engine`: The WASM engine instance
/// 
/// # Example
/// 
/// ```rust
/// use dlink_wm::host_import::init_store_with_wasi;
/// 
/// let (store, wasi_ctx, engine) = init_store_with_wasi();
/// ```
pub fn init_store_with_wasi() -> (Store<WasiCtx>, WasiCtx, Engine) {
    let engine = Engine::default();
    let wasi_ctx = WasiCtxBuilder::new()
        .inherit_stdio()
        .build();
    let store = Store::new(&engine, wasi_ctx.clone());
    (store, wasi_ctx, engine)
}

/// # Host Memory Allocation
/// 
/// Allocates memory in the host for use by WASM modules.
/// 
/// # Parameters
/// 
/// - `caller`: WASM caller context
/// - `size`: Size of memory to allocate in bytes
/// 
/// # Returns
/// 
/// Pointer to the allocated memory block, or `-1` if allocation failed.
/// 
/// # Notes
/// 
/// This is a simplified implementation for demonstration purposes. In a production
/// environment, a proper memory allocator should be used.
pub fn host_malloc(
    mut caller: Caller<'_, WasiCtx>,
    _size: i32,
) -> i32 {
    // Get WASM memory
    let _memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
        Some(mem) => mem,
        None => return -1,
    };

    // Simplified allocation strategy: fixed address allocation
    // In real applications, use a proper memory allocator
    let alloc_ptr = 0x100000; // Start allocation from this address
    alloc_ptr
}

/// # Host Memory Free
/// 
/// Frees memory allocated by `host_malloc`.
/// 
/// # Parameters
/// 
/// - `caller`: WASM caller context
/// - `ptr`: Pointer to the memory block to free
/// 
/// # Notes
/// 
/// This is a no-op implementation for demonstration purposes. In a production
/// environment, a proper memory allocator should be used.
pub fn host_free(
    _caller: Caller<'_, WasiCtx>,
    _ptr: i32,
) {
    // Simplified implementation: no-op
    // In real applications, use a proper memory allocator to free memory
}

/// # Create and Configure Linker
/// 
/// Creates a new linker and configures it with the necessary host imports,
/// including the universal invocation interface and memory management functions.
/// 
/// # Parameters
/// 
/// - `engine`: WASM engine instance to use for linker creation
/// 
/// # Returns
/// 
/// A configured linker ready to instantiate WASM modules.
/// 
/// # Example
/// 
/// ```rust
/// use dlink_wm::host_import::create_dlinkwm_linker;
/// use wasmtime::{Engine, Store};
/// use wasmtime_wasi::WasiCtx;
/// use anyhow::Result;
/// 
/// fn example() -> Result<()> {
///     let engine = Engine::default();
///     let linker = create_dlinkwm_linker(&engine)?;
///     Ok(())
/// }
/// ```
pub fn create_dlinkwm_linker(engine: &Engine) -> Result<Linker<WasiCtx>> {
    // Create a new linker instance
    let mut linker = Linker::new(engine);

    // Register WASI imports
    wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;

    // Register host import functions
    linker.func_wrap("dlinkwm_host", "universal_invoke", universal_invoke)?;
    linker.func_wrap("dlinkwm_host", "host_malloc", host_malloc)?;
    linker.func_wrap("dlinkwm_host", "host_free", host_free)?;

    Ok(linker)
}

// -------------------------- Internal Helper Structures --------------------------
// All helper structures have been removed as they are not currently used
