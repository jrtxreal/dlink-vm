use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, RwLock};
use notify::{Watcher, RecursiveMode, RecommendedWatcher, EventKind, Config};
use anyhow::Result;
use std::thread;
use std::sync::mpsc::channel;

/// # DlinkWM Configuration
/// 
/// Represents the configuration for DlinkWM, defining which functions can be called
/// from which WASM files.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DlinkWMConfig {
    /// # Per-file Entry Functions Mapping
    /// 
    /// Defines specific entry functions for different WASM files.
    /// - **Key**: WASM file path (relative or absolute)
    /// - **Value**: List of entry functions to try for this specific file
    /// 
    /// Example TOML configuration:
    /// ```toml
    /// [entry_functions]
    /// "wasm/wasm_test.wasm" = ["dlinkwm_print_hello_wasm", "dlinkwm_test_host_methods"]
    /// "wasm/hello_simple.wasm" = ["dlinkwm_simple_entry"]
    /// ```
    pub entry_functions: std::collections::HashMap<String, Vec<String>>,
}

impl Default for DlinkWMConfig {
    /// Creates a default configuration with empty entry functions mapping.
    fn default() -> Self {
        Self {
            entry_functions: std::collections::HashMap::new(),
        }
    }
}

impl DlinkWMConfig {
    /// Loads configuration from a TOML file.
    /// 
    /// # Parameters
    /// 
    /// - `path`: Path to the TOML configuration file
    /// 
    /// # Returns
    /// 
    /// A `DlinkWMConfig` instance loaded from the file, or a default instance if the file doesn't exist.
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - The file exists but cannot be read
    /// - The file contains invalid TOML
    /// - The TOML structure doesn't match the expected format
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        if !path.as_ref().exists() {
            // Return default config if file doesn't exist
            Ok(DlinkWMConfig::default())
        } else {
            let content = std::fs::read_to_string(path)?;
            let config: DlinkWMConfig = toml::from_str(&content)?;
            Ok(config)
        }
    }

    /// Saves the configuration to a TOML file.
    /// 
    /// # Parameters
    /// 
    /// - `path`: Path where the configuration should be saved
    /// 
    /// # Returns
    /// 
    /// `Ok(())` if the configuration is saved successfully, otherwise an error.
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - The file cannot be created or written to
    /// - The configuration cannot be serialized to TOML
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let toml_str = toml::to_string_pretty(self)?;
        let mut file = File::create(path)?;
        file.write_all(toml_str.as_bytes())?;
        Ok(())
    }
}

/// # Dynamic Configuration Manager
/// 
/// Thread-safe configuration manager with hot reload support. This structure
/// monitors the configuration file for changes and automatically reloads it.
/// 
/// The `DynamicConfig` provides safe access to the configuration across multiple threads
/// and ensures that all threads see the latest configuration after a reload.
#[derive(Debug)]
pub struct DynamicConfig {
    /// Thread-safe configuration storage
    config: Arc<RwLock<DlinkWMConfig>>,
    /// Path to the configuration file being monitored
    config_path: String,
    /// File watcher for detecting configuration changes
    watcher: Option<RecommendedWatcher>,
}

impl DynamicConfig {
    /// Creates a new dynamic configuration manager.
    /// 
    /// # Parameters
    /// 
    /// - `config_path`: Path to the TOML configuration file to load and monitor
    /// 
    /// # Returns
    /// 
    /// A new instance of `DynamicConfig` initialized with the configuration from the file.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the initial configuration cannot be loaded.
    pub fn new(config_path: &str) -> Result<Self> {
        // Load initial configuration from file
        let config = DlinkWMConfig::load_from_file(config_path)?;
        
        // Create the dynamic config instance
        let dynamic_config = Self {
            config: Arc::new(RwLock::new(config)),
            config_path: config_path.to_string(),
            watcher: None,
        };
        
        Ok(dynamic_config)
    }

    /// Starts watching the configuration file for changes.
    /// 
    /// This function spawns a background thread that:
    /// 1. Watches the configuration file for modifications
    /// 2. Reloads the configuration when changes are detected
    /// 3. Updates the thread-safe configuration storage
    /// 
    /// # Returns
    /// 
    /// `Ok(())` if the watcher is started successfully, otherwise an error.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the file watcher cannot be created or started.
    pub fn start_watching(&mut self) -> Result<()> {
        // Check if the config file exists
        let config_file = Path::new(&self.config_path);
        if !config_file.exists() {
            log::warn!("[Config] Warning: Config file does not exist: {:?}", config_file);
            return Ok(()); // Don't watch non-existent files
        }
        
        // Create communication channel for watcher events
        let (tx, rx) = channel();
        
        // Create a watcher with default configuration
        let mut watcher = RecommendedWatcher::new(
            move |res| {
                if let Err(e) = tx.send(res) {
                    log::error!("[Config] Failed to send event: {}", e);
                }
            },
            Config::default()
        )?;
        
        // Watch the config file with non-recursive mode
        if let Err(e) = watcher.watch(config_file, RecursiveMode::NonRecursive) {
            log::error!("[Config] Failed to watch config file: {}", e);
            return Ok(()); // Continue without watching if we can't
        }
        
        log::info!("[Config] Successfully started watching config file: {:?}", config_file);
        
        // Clone references for the watcher thread
        let config = Arc::clone(&self.config);
        let config_path = self.config_path.clone();
        
        // Start a thread to handle configuration change events
        thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(event) => {
                        match event {
                            Ok(event) => {
                                // Only handle file modification events
                                if let EventKind::Modify(_) = event.kind {
                                    log::info!("[Config] Detected config file change, reloading...");
                                    
                                    // Reload the configuration
                                    match DlinkWMConfig::load_from_file(&config_path) {
                                        Ok(new_config) => {
                                            let mut current_config = config.write().unwrap();
                                            *current_config = new_config;
                                            log::info!("[Config] Config reloaded successfully");
                                            log::debug!("[Config] New entry functions: {:?}", current_config.entry_functions);
                                        }
                                        Err(e) => {
                                            log::error!("[Config] Failed to reload config: {}", e);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("[Config] Event error: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("[Config] Watcher error: {}", e);
                        break; // Exit loop if channel is closed
                    }
                }
            }
        });
        
        // Store the watcher
        self.watcher = Some(watcher);
        
        Ok(())
    }

    /// Gets a reference to the current thread-safe configuration.
    /// 
    /// This returns an `Arc<RwLock<DlinkWMConfig>>` which allows multiple threads to
    /// safely read the configuration while ensuring that writes (during reloads) are
    /// properly synchronized.
    /// 
    /// # Returns
    /// 
    /// An `Arc<RwLock<DlinkWMConfig>>` reference to the current configuration.
    pub fn get_config(&self) -> Arc<RwLock<DlinkWMConfig>> {
        Arc::clone(&self.config)
    }
    
    /// Gets the list of allowed entry functions for a specific WASM file.
    /// 
    /// This function checks the configuration for entry functions defined for the
    /// specified WASM file and returns them if found, otherwise returns an empty vector.
    /// 
    /// # Parameters
    /// 
    /// - `file_path`: Path to the WASM file to get entry functions for
    /// 
    /// # Returns
    /// 
    /// A vector of allowed entry function names for the specified WASM file.
    pub fn get_entry_functions_for_file(&self, file_path: &str) -> Vec<String> {
        let config_read = self.config.read().unwrap();
        
        // Try to get entry functions for the specific file
        if let Some(functions) = config_read.entry_functions.get(file_path) {
            functions.clone()
        } else {
            // Return empty vector if no entry functions are defined for this file
            Vec::new()
        }
    }
}

/// Gets the default configuration file path.
/// 
/// # Returns
/// 
/// The default configuration file path, which is "dlinkwm.toml" in the current directory.
pub fn get_default_config_path() -> String {
    "dlinkwm.toml".to_string()
}

/// Creates a default configuration file if it doesn't exist.
/// 
/// This function checks if the default configuration file exists, and if not,
/// creates it with an empty configuration.
/// 
/// # Returns
/// 
/// `Ok(())` if the operation is successful, otherwise an error.
/// 
/// # Errors
/// 
/// Returns an error if the default configuration file cannot be created.
pub fn create_default_config_if_missing() -> Result<()> {
    let config_path = get_default_config_path();
    
    // Create default config if it doesn't exist
    let config_file_path = Path::new(&config_path);
    if !config_file_path.exists() {
        let default_config = DlinkWMConfig::default();
        default_config.save_to_file(config_file_path)?;
        log::info!("[Config] Created default config file: {:?}", config_file_path);
    }
    
    Ok(())
}
