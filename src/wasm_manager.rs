use wasmtime::{Module, Instance, Store};
use wasmtime_wasi::{WasiCtx};
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use notify::Watcher;
use std::thread;
use crate::host_import::{init_store_with_wasi, create_dlinkwm_linker};
use crate::config::DynamicConfig;
use anyhow::{anyhow, Result as AnyResult};

/// # WASM Instance Cache
/// 
/// Manages the caching of WASM modules and instances to reduce compilation and instantiation overhead.
/// 
/// The cache maintains two levels of caching:
/// 1. **Module Cache**: Stores compiled WASM modules, which can be reused to instantiate multiple instances
/// 2. **Instance Cache**: Stores instantiated WASM modules, including their store context
/// 
/// This structure is thread-safe and can be shared across multiple threads.
pub struct WasmInstanceCache {
    /// Cache of compiled WASM modules (reduces compilation overhead)
    module_cache: Arc<RwLock<HashMap<String, Module>>>,
    /// Cache of instantiated WASM modules (each file has one instance)
    instance_cache: Arc<RwLock<HashMap<String, Arc<RwLock<(Instance, Store<WasiCtx>)>>>>>,
}

impl WasmInstanceCache {
    /// Creates a new WASM instance cache.
    /// 
    /// # Returns
    /// 
    /// A new instance of `WasmInstanceCache` with empty caches.
    pub fn new() -> Self {
        Self {
            module_cache: Arc::new(RwLock::new(HashMap::new())),
            instance_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Loads and instantiates a WASM file.
    /// 
    /// This function:
    /// 1. Checks if the instance is already in cache and returns it if found
    /// 2. If not in cache, reads the WASM file content
    /// 3. Checks if the module is already compiled and cached
    /// 4. If not, compiles the module and caches it
    /// 5. Instantiates the module and caches the instance
    /// 
    /// # Parameters
    /// 
    /// - `wasm_path`: Path to the WASM file to load and instantiate
    /// 
    /// # Returns
    /// 
    /// An `Arc<RwLock<(Instance, Store<WasiCtx>)>>` containing the instantiated WASM module
    /// and its associated store context.
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - The WASM file cannot be read
    /// - The module cannot be compiled
    /// - The module cannot be instantiated
    pub fn load_and_instantiate(&self, wasm_path: &str) -> AnyResult<Arc<RwLock<(Instance, Store<WasiCtx>)>>> {
        let wasm_path_str = wasm_path.to_string();
        
        // Try to get instance from cache
        {
            let cache_read = self.instance_cache.read().unwrap();
            if let Some(instance_store) = cache_read.get(&wasm_path_str) {
                return Ok(instance_store.clone());
            }
        }
        
        // Read WASM file content
        let mut file = File::open(wasm_path)?;
        let mut wasm_bytes = Vec::new();
        file.read_to_end(&mut wasm_bytes)?;
        
        // Initialize Store and WASI context
        let (mut store, _, engine) = init_store_with_wasi();
        
        // Try to get module from cache
        let module = {
            // First check cache with minimal read lock scope
            {
                let cache_read = self.module_cache.read().unwrap();
                if let Some(cached_module) = cache_read.get(&wasm_path_str) {
                    cached_module.clone()
                } else {
                    // If not in cache, release read lock and compile
                    drop(cache_read);
                    
                    // Compile WASM module
                    let module = Module::new(&engine, &wasm_bytes)?;
                    self.module_cache.write().unwrap().insert(wasm_path_str.clone(), module.clone());
                    module
                }
            }
        };
        
        // Create and configure Linker with host imports
        let linker = create_dlinkwm_linker(&engine)?;

        // Instantiate module
        let instance = linker.instantiate(&mut store, &module)?;
        
        // Create thread-safe wrapper for instance and store
        let instance_store = Arc::new(RwLock::new((instance, store)));
        
        // Cache instance and Store
        self.instance_cache.write().unwrap().insert(wasm_path_str, instance_store.clone());
        Ok(instance_store)
    }

    /// Clears the cache for a specific WASM file.
    /// 
    /// This removes both the compiled module and the instantiated instance from cache.
    /// 
    /// # Parameters
    /// 
    /// - `wasm_path`: Path to the WASM file whose cache should be cleared
    pub fn clear_cache(&self, wasm_path: &str) {
        let wasm_path_str = wasm_path.to_string();
        self.module_cache.write().unwrap().remove(&wasm_path_str);
        self.instance_cache.write().unwrap().remove(&wasm_path_str);
    }

    /// Triggers a hot reload for a specific WASM file.
    /// 
    /// This function:
    /// 1. Clears the cache for the specified WASM file
    /// 2. Reloads and reinstantiates the file
    /// 3. Returns the newly instantiated module
    /// 
    /// # Parameters
    /// 
    /// - `wasm_path`: Path to the WASM file to hot reload
    /// 
    /// # Returns
    /// 
    /// An `Arc<RwLock<(Instance, Store<WasiCtx>)>>` containing the newly instantiated WASM module
    /// and its associated store context.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the WASM file cannot be reloaded and reinstantiated.
    pub fn hot_reload(&self, wasm_path: &str) -> AnyResult<Arc<RwLock<(Instance, Store<WasiCtx>)>>> {
        // Clear cache to ensure fresh reload
        self.clear_cache(wasm_path);
        // Reload and instantiate
        self.load_and_instantiate(wasm_path)
    }
}

/// # WASM Hot Reloader
/// 
/// Monitors WASM files for changes and automatically triggers hot reloads when they change.
/// 
/// This structure watches a directory for changes to `.wasm` files and automatically
/// calls `hot_reload` on the associated `WasmInstanceCache` when changes are detected.
/// 
/// The hot reloader runs in a separate background thread, allowing the main application
/// to continue executing while monitoring for changes.
pub struct WasmHotReloader {
    /// Reference to the WASM instance cache to reload modules from
    instance_cache: Arc<WasmInstanceCache>,
    /// Directory path to watch for WASM file changes
    watch_path: String,
}

impl WasmHotReloader {
    /// Creates a new WASM hot reload manager.
    /// 
    /// # Parameters
    /// 
    /// - `instance_cache`: Reference to the WASM instance cache to use for reloading
    /// - `watch_path`: Directory path to watch for WASM file changes
    /// 
    /// # Returns
    /// 
    /// A new instance of `WasmHotReloader` configured to watch the specified directory.
    pub fn new(instance_cache: Arc<WasmInstanceCache>, watch_path: &str) -> Self {
        Self {
            instance_cache,
            watch_path: watch_path.to_string(),
        }
    }

    /// Starts the hot reload monitoring thread.
    /// 
    /// This function spawns a background thread that:
    /// 1. Watches the specified directory for file changes
    /// 2. Detects when `.wasm` files are modified
    /// 3. Automatically triggers hot reload for the modified files
    /// 
    /// The monitoring continues until the application exits or the watcher encounters an error.
    pub fn start(&self) {
        // Create communication channel for watcher events
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = notify::RecommendedWatcher::new(tx, notify::Config::default()).unwrap();
        
        // Recursively watch the directory
        watcher.watch(std::path::Path::new(&self.watch_path), notify::RecursiveMode::Recursive).unwrap();
        
        let instance_cache_clone = self.instance_cache.clone();
        let watch_path_clone = self.watch_path.clone();
        
        // Start monitoring thread
        thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(event_result) => match event_result {
                        Ok(event) => {
                            // Handle file modification events
                            if let notify::EventKind::Modify(notify::event::ModifyKind::Data(_)) = event.kind {
                                for path in event.paths {
                                    // Check if the modified file is a WASM file
                                    if let Some(ext) = path.extension() {
                                        if ext == "wasm" {
                                            let wasm_path = path.to_string_lossy().to_string();
                                            log::info!("[HotReload] Detected WASM change: {}", wasm_path);
                                            
                                            // Trigger hot reload
                                            match instance_cache_clone.hot_reload(&wasm_path) {
                                                Ok(_) => log::info!("[HotReload] Successfully hot reloaded: {}", wasm_path),
                                                Err(e) => log::error!("[HotReload] Failed to hot reload: {}, error: {}", wasm_path, e),
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        Err(e) => {
                            log::error!("[HotReload] Watcher error: {}", e);
                            break; // Exit loop on watcher error
                        },
                    },
                    Err(_) => {
                        // Channel closed, exit loop
                        break;
                    }
                }
            }
        });
        
        log::info!("[HotReload] Started watching: {}", watch_path_clone);
    }
}

/// # Load WASM Instance (Simplified API)
/// 
/// A convenience function that loads and instantiates a WASM file using the provided cache.
/// 
/// This is a simple wrapper around `WasmInstanceCache::load_and_instantiate`.
/// 
/// # Parameters
/// 
/// - `wasm_path`: Path to the WASM file to load and instantiate
/// - `instance_cache`: Reference to the WASM instance cache to use
/// 
/// # Returns
/// 
/// An `Arc<RwLock<(Instance, Store<WasiCtx>)>>` containing the instantiated WASM module
/// and its associated store context.
/// 
/// # Errors
/// 
/// Returns an error if the WASM file cannot be loaded and instantiated.
pub fn load_wasm_instance(wasm_path: &str, instance_cache: &Arc<WasmInstanceCache>) -> AnyResult<Arc<RwLock<(Instance, Store<WasiCtx>)>>> {
    instance_cache.load_and_instantiate(wasm_path)
}

/// # Call WASM Function with Configuration Validation
/// 
/// Safely calls a WASM function, validating that it's configured as an allowed entry function.
/// 
/// This function provides a safe way to call WASM functions by:
/// 1. Checking if the function is in the allowed entry functions list for the WASM file
/// 2. Clearing the cache to ensure the latest WASM file is used
/// 3. Loading and instantiating the WASM module
/// 4. Calling the specified function with proper error handling
/// 5. Handling both string-returning and void functions
/// 
/// # Parameters
/// 
/// - `wasm_path`: Path to the WASM file containing the function
/// - `func_name`: Name of the function to call
/// - `instance_cache`: Reference to the WASM instance cache to use
/// - `dynamic_config`: Reference to the dynamic configuration to check function permissions against
/// 
/// # Returns
/// 
/// `Ok(())` if the function call is successful, otherwise an error.
/// 
/// # Errors
/// 
/// Returns an error if:
/// - The function is not configured as an entry function for the WASM file
/// - The WASM file cannot be loaded or instantiated
/// - The function is not found in the WASM module
/// - The function is not a function type
/// - The function has an incompatible signature
/// - The function call fails during execution
pub fn call_wasm_function(
    wasm_path: &str,
    func_name: &str,
    instance_cache: &Arc<WasmInstanceCache>,
    dynamic_config: &DynamicConfig
) -> AnyResult<()> {
    // Get allowed entry functions for the specified WASM file
    let entry_functions = dynamic_config.get_entry_functions_for_file(wasm_path);
    
    // Validate that the requested function is in the allowed list
    if !entry_functions.contains(&func_name.to_string()) {
        return Err(anyhow!(
            "Function '{}' is not configured as an entry function for WASM file '{}'. Allowed functions: {:?}",
            func_name,
            wasm_path,
            entry_functions
        ));
    }
    
    // Clear cache to ensure we use the latest WASM file
    instance_cache.clear_cache(wasm_path);
    
    // Load and instantiate the WASM module
    let instance_store = instance_cache.load_and_instantiate(wasm_path)?;
    
    // Get exclusive access to the instance and store
    let mut guard = instance_store.write().unwrap();
    let (ref mut instance, ref mut store) = *guard;
    
    // Try to call the specified function
    if let Some(extern_val) = instance.get_export(&mut *store, func_name) {
        if let Some(func) = extern_val.into_func() {
            // First try as function returning a string pointer (i32)
            match func.typed::<(), i32>(&mut *store) {
                Ok(test_func) => {
                    let result_ptr = test_func.call(&mut *store, ())?;
                    println!("✅ WASM function '{}' called successfully", func_name);
                    println!("   Raw return value (pointer): {:#018x}", result_ptr);
                    
                    // Read the returned string from memory if available
                    if let Some(memory) = instance.get_memory(&mut *store, "memory") {
                        let mut buffer = Vec::new();
                        let mut offset = 0;
                        
                        // Read bytes until null terminator is found
                        loop {
                            let mut byte_buffer = [0u8; 1];
                            memory.read(&mut *store, result_ptr as usize + offset, &mut byte_buffer)?;
                            let byte = byte_buffer[0];
                            
                            if byte == 0 {
                                break; // Null terminator found
                            }
                            buffer.push(byte);
                            offset += 1;
                        }
                        
                        // Process the read buffer
                        if !buffer.is_empty() {
                            let result = String::from_utf8(buffer)?;
                            println!("   Return value: '{}'", result);
                            println!("   String length: {} bytes", result.len());
                        } else {
                            println!("   Returned empty string");
                        }
                    }
                    Ok(())
                },
                Err(_) => {
                    // If that fails, try as a void function (no return value)
                    match func.typed::<(), ()>(&mut *store) {
                        Ok(test_func) => {
                            test_func.call(&mut *store, ())?;
                            println!("✅ WASM function '{}' called successfully (no return value)", func_name);
                            Ok(())
                        },
                        Err(err) => {
                            Err(anyhow!("Cannot call function '{}': incompatible signature, error: {:?}", func_name, err))
                        }
                    }
                }
            }
        } else {
            Err(anyhow!("Export '{}' is not a function", func_name))
        }
    } else {
        Err(anyhow!("Export '{}' not found in WASM module", func_name))
    }
}
