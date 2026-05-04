//! WebAssembly sandbox runtime
//!
//! Executes untrusted WASM modules in an isolated wasmtime engine with:
//!   - No WASI / filesystem / network access (linker has no host imports)
//!   - Fuel-based instruction budget (1 000 000 ops ≈ a few ms of pure compute)
//!   - Hard 10-second wall-clock timeout via tokio
//!
//! Usage:
//!   let sandbox = WasmSandbox::new()?;
//!   let result  = sandbox.execute(&wasm_bytes, "run", &[]).await?;

use thiserror::Error;
use wasmtime::{Config, Engine, Linker, Module, Store, Val};

#[derive(Error, Debug)]
pub enum SandboxError {
    #[error("Failed to load WASM module: {0}")]
    LoadError(String),

    #[error("Execution failed: {0}")]
    ExecutionError(String),

    #[error("Capability denied: {0}")]
    CapabilityDenied(String),

    #[error("Sandbox timed out (10 s wall clock / fuel budget exceeded)")]
    Timeout,
}

/// A sandboxed WASM runtime.
/// Each `execute()` call gets a fresh `Store` so state never leaks between calls.
pub struct WasmSandbox {
    engine: Engine,
}

impl WasmSandbox {
    /// Initialise the wasmtime engine with fuel metering enabled.
    pub fn new() -> Result<Self, SandboxError> {
        let mut cfg = Config::new();
        cfg.consume_fuel(true);
        let engine =
            Engine::new(&cfg).map_err(|e| SandboxError::LoadError(e.to_string()))?;
        Ok(Self { engine })
    }

    /// Execute `function` from the compiled `module_bytes`.
    ///
    /// `_args` is reserved for future ABI work (shared memory / WASI).
    /// Currently the function must take no arguments and return zero or more
    /// integer/float values, which are serialised as little-endian bytes.
    pub async fn execute(
        &self,
        module_bytes: &[u8],
        function: &str,
        _args: &[u8],
    ) -> Result<Vec<u8>, SandboxError> {
        let engine = self.engine.clone();
        let module_bytes = module_bytes.to_vec();
        let function = function.to_string();

        let blocking = tokio::task::spawn_blocking(move || {
            // Compile module — validates WASM binary
            let module = Module::from_binary(&engine, &module_bytes)
                .map_err(|e| SandboxError::LoadError(e.to_string()))?;

            // Empty linker — no host imports, no WASI. The module cannot call
            // out to the host environment at all.
            let linker: Linker<()> = Linker::new(&engine);

            let mut store = Store::new(&engine, ());
            // Fuel budget: 1 000 000 instructions ≈ a few ms of compute.
            // Exceeding this traps with OutOfFuel rather than hanging.
            store
                .set_fuel(1_000_000)
                .map_err(|e| SandboxError::ExecutionError(e.to_string()))?;

            let instance = linker
                .instantiate(&mut store, &module)
                .map_err(|e| SandboxError::ExecutionError(e.to_string()))?;

            let func = instance
                .get_func(&mut store, &function)
                .ok_or_else(|| {
                    SandboxError::ExecutionError(format!(
                        "function '{}' not exported by WASM module",
                        function
                    ))
                })?;

            let n_results = func.ty(&store).results().len();
            let mut results = vec![Val::I32(0); n_results];

            func.call(&mut store, &[], &mut results)
                .map_err(|e| SandboxError::ExecutionError(e.to_string()))?;

            // Serialise results as little-endian bytes
            let mut out = Vec::new();
            for val in &results {
                match val {
                    Val::I32(v) => out.extend_from_slice(&v.to_le_bytes()),
                    Val::I64(v) => out.extend_from_slice(&v.to_le_bytes()),
                    Val::F32(v) => out.extend_from_slice(&v.to_le_bytes()),
                    Val::F64(v) => out.extend_from_slice(&v.to_le_bytes()),
                    _ => {}
                }
            }
            Ok(out)
        });

        // Wall-clock safety net — if the fuel budget is somehow bypassed
        // (e.g. a very tight loop that the metering misses) we still terminate.
        tokio::time::timeout(std::time::Duration::from_secs(10), blocking)
            .await
            .map_err(|_| SandboxError::Timeout)?
            .map_err(|e| SandboxError::ExecutionError(e.to_string()))?
    }
}
