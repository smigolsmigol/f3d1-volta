//! f3d1-volta-core: Phase 0 spike.
//!
//! Goal: prove the architectural pattern that the keystone depends on.
//! A Rust runtime spawns N tokio tasks; each task acquires the GIL,
//! calls a Python callable, releases the GIL between tasks so they
//! actually run in parallel. If this works cleanly, the rest of the
//! 24-week plan stands. If it deadlocks, the plan dies on day 1.
//!
//! Critical guarantees we test:
//!   1. GIL acquire / release happens correctly across tokio task
//!      boundaries (no deadlock under contention).
//!   2. Python exceptions propagate back as Rust errors.
//!   3. Parallel branches actually parallelize - the wall-clock for N
//!      blocking-Python-callbacks of D ms each is closer to D than to
//!      N*D when the callback is `time.sleep` style (the GIL is
//!      released during the sleep).
//!   4. No leaks of Python objects across the boundary (PyObject Drop
//!      runs while the GIL is held).

use pyo3::prelude::*;
use std::time::Instant;
use tokio::runtime::Runtime;

/// Run `callback(input)` for each `input` in `inputs`, in parallel
/// tokio tasks. Each task acquires the GIL only when it calls the
/// callback, releases between calls.
///
/// Returns the results in the same order as inputs, or the first
/// Python exception encountered.
pub fn run_parallel(
    callback: PyObject,
    inputs: Vec<PyObject>,
) -> PyResult<Vec<PyObject>> {
    // Build a multi-thread tokio runtime locally to the call. In the
    // real keystone this becomes a long-lived runtime field on the
    // Graph. The pattern transfers.
    let rt = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("tokio runtime: {e}"))
    })?;

    let n = inputs.len();
    rt.block_on(async move {
        let mut handles = Vec::with_capacity(n);
        for input in inputs {
            // Clone the callback PyObject for each task. PyObject is
            // cheap to clone (it's a smart pointer / Py<PyAny>).
            let cb = Python::with_gil(|py| callback.clone_ref(py));
            let handle = tokio::task::spawn_blocking(move || {
                Python::with_gil(|py| -> PyResult<PyObject> {
                    let result = cb.call1(py, (input,))?;
                    Ok(result)
                })
            });
            handles.push(handle);
        }

        let mut out = Vec::with_capacity(n);
        for handle in handles {
            let r = handle.await.map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("tokio join: {e}"))
            })??;
            out.push(r);
        }
        Ok(out)
    })
}

/// Same as run_parallel but also returns the wall-clock so callers
/// can compare against the sequential baseline.
pub fn run_parallel_timed(
    callback: PyObject,
    inputs: Vec<PyObject>,
) -> PyResult<(Vec<PyObject>, f64)> {
    let t0 = Instant::now();
    let out = run_parallel(callback, inputs)?;
    Ok((out, t0.elapsed().as_secs_f64()))
}

#[pymodule]
fn _f3d1_volta_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    #[pyfn(m)]
    fn run_parallel_py(
        py: Python<'_>,
        callback: PyObject,
        inputs: Vec<PyObject>,
    ) -> PyResult<Vec<PyObject>> {
        py.allow_threads(|| run_parallel(callback, inputs))
    }

    #[pyfn(m)]
    fn run_parallel_timed_py(
        py: Python<'_>,
        callback: PyObject,
        inputs: Vec<PyObject>,
    ) -> PyResult<(Vec<PyObject>, f64)> {
        py.allow_threads(|| run_parallel_timed(callback, inputs))
    }
    Ok(())
}

// Tests live in `tests/test_phase0_spike.py` and run as a Python extension
// via maturin. Embedded-Python `cargo test` mode hangs on Windows; the real
// production shape is "Python imports the .pyd / .so", which is what we test.
