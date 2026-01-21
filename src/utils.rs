//! # Utility Functions
//! 
//! This module provides utility functions for working with WASM memory and serialization.
//! It includes functions for reading and writing to WASM linear memory, as well as
//! serialization and deserialization helpers.

use serde::{Serialize, Deserialize};
use wasmtime::{Memory, AsContext, AsContextMut};
use anyhow::Result;

/// # Read from WASM Memory
/// 
/// Reads a byte array from WASM linear memory at the specified address and length.
/// 
/// # Parameters
/// 
/// - `memory`: Reference to the WASM memory instance
/// - `store`: WASM context used to access memory
/// - `ptr`: Pointer to the start of the data in WASM memory
/// - `len`: Length of the data to read in bytes
/// 
/// # Returns
/// 
/// A `Result` containing the read byte array, or an error if the read operation fails.
/// 
/// # Example
/// 
/// ```rust
/// use wasmtime::{Memory, Store};
/// use dlink_wm::utils::read_wasm_memory;
/// 
/// fn example(memory: &Memory, store: &Store<()>, ptr: i32, len: i32) -> anyhow::Result<()> {
///     let data = read_wasm_memory(memory, store, ptr, len)?;
///     println!("Read {} bytes from WASM memory", data.len());
///     Ok(())
/// }
/// ```
pub fn read_wasm_memory(memory: &Memory, store: impl AsContext, ptr: i32, len: i32) -> Result<Vec<u8>> {
    let ptr = ptr as usize;
    let len = len as usize;
    let mut buffer = vec![0u8; len];
    memory.read(store, ptr, &mut buffer)?;
    Ok(buffer)
}

/// # Write to WASM Memory
/// 
/// Writes a byte array to WASM linear memory at the specified address.
/// 
/// # Parameters
/// 
/// - `memory`: Reference to the WASM memory instance
/// - `store`: Mutable WASM context used to access memory
/// - `ptr`: Pointer to write the data to in WASM memory
/// - `data`: Data to write to WASM memory
/// 
/// # Returns
/// 
/// A `Result` indicating success or failure of the write operation.
/// 
/// # Example
/// 
/// ```rust
/// use wasmtime::{Memory, Store};
/// use dlink_wm::utils::write_wasm_memory;
/// 
/// fn example(memory: &Memory, store: &mut Store<()>, ptr: i32, data: &[u8]) -> anyhow::Result<()> {
///     write_wasm_memory(memory, store, ptr, data)?;
///     println!("Wrote {} bytes to WASM memory", data.len());
///     Ok(())
/// }
/// ```
pub fn write_wasm_memory(memory: &Memory, store: impl AsContextMut, ptr: i32, data: &[u8]) -> Result<()> {
    let ptr = ptr as usize;
    memory.write(store, ptr, data)?;
    Ok(())
}

/// # Deserialize from WASM Memory
/// 
/// Deserializes JSON data from WASM linear memory into a Rust structure.
/// 
/// # Parameters
/// 
/// - `memory`: Reference to the WASM memory instance
/// - `store`: WASM context used to access memory
/// - `ptr`: Pointer to the start of the serialized JSON data in WASM memory
/// - `len`: Length of the serialized JSON data in bytes
/// 
/// # Returns
/// 
/// A `Result` containing the deserialized Rust structure, or an error if the deserialization fails.
/// 
/// # Type Parameters
/// 
/// - `T`: Type to deserialize into, must implement `serde::Deserialize`
/// 
/// # Example
/// 
/// ```rust
/// use serde::{Serialize, Deserialize};
/// use wasmtime::{Memory, Store};
/// use dlink_wm::utils::deserialize_from_wasm;
/// 
/// #[derive(Debug, Deserialize)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
/// 
/// fn example(memory: &Memory, store: &Store<()>, ptr: i32, len: i32) -> anyhow::Result<()> {
///     let person: Person = deserialize_from_wasm(memory, store, ptr, len)?;
///     println!("Deserialized: {:?}", person);
///     Ok(())
/// }
/// ```
pub fn deserialize_from_wasm<T: for<'a> Deserialize<'a>>(
    memory: &Memory,
    store: impl AsContext,
    ptr: i32,
    len: i32
) -> Result<T> {
    let buffer = read_wasm_memory(memory, store, ptr, len)?;
    let result = serde_json::from_slice(&buffer)?;
    Ok(result)
}

/// # Serialize to WASM Memory
/// 
/// Serializes a Rust structure to JSON and writes it to WASM linear memory.
/// 
/// # Parameters
/// 
/// - `memory`: Reference to the WASM memory instance
/// - `store`: Mutable WASM context used to access memory
/// - `ptr`: Pointer to write the serialized JSON data to in WASM memory
/// - `data`: Rust structure to serialize and write
/// 
/// # Returns
/// 
/// A `Result` containing the number of bytes written to WASM memory, or an error if the serialization fails.
/// 
/// # Type Parameters
/// 
/// - `T`: Type to serialize, must implement `serde::Serialize`
/// 
/// # Example
/// 
/// ```rust
/// use serde::{Serialize, Deserialize};
/// use wasmtime::{Memory, Store};
/// use dlink_wm::utils::serialize_to_wasm;
/// 
/// #[derive(Debug, Serialize)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
/// 
/// fn example(memory: &Memory, store: &mut Store<()>, ptr: i32) -> anyhow::Result<()> {
///     let person = Person { name: "Alice".to_string(), age: 30 };
///     let bytes_written = serialize_to_wasm(memory, store, ptr, &person)?;
///     println!("Serialized and wrote {} bytes to WASM memory", bytes_written);
///     Ok(())
/// }
/// ```
pub fn serialize_to_wasm<T: Serialize>(
    memory: &Memory,
    store: impl AsContextMut,
    ptr: i32,
    data: &T
) -> Result<usize> {
    let buffer = serde_json::to_vec(data)?;
    write_wasm_memory(memory, store, ptr, &buffer)?;
    Ok(buffer.len())
}


