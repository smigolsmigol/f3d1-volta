"""Phase 0 spike: validate the pyo3 + tokio architectural pattern.

The day-1 kill criterion. If parallel_sleeps_actually_parallelize passes,
the 24-week f3d1-volta plan stands. If it fails, the plan dies.
"""

from __future__ import annotations

import time

import pytest

from f3d1_volta import run_parallel_py, run_parallel_timed_py


def test_callback_returns_value() -> None:
    """Smoke: Rust spawns a tokio task, calls Python lambda, returns result."""
    out = run_parallel_py(lambda x: x * 2, [3])
    assert out == [6]


def test_python_exception_propagates() -> None:
    """A Python exception inside the callback surfaces as a Python exception
    on the Rust -> Python boundary."""
    with pytest.raises(ZeroDivisionError):
        run_parallel_py(lambda x: 1 / 0, [1])


def test_multiple_inputs_return_in_order() -> None:
    """Result list aligns with input list."""
    out = run_parallel_py(lambda x: x + 100, [1, 2, 3, 4, 5])
    assert out == [101, 102, 103, 104, 105]


def test_parallel_sleeps_actually_parallelize() -> None:
    """The big one. Parallelism is real.

    A callback that does time.sleep(0.1) - which releases the GIL during
    the sleep - called 5 times in parallel should finish in ~0.1s, not
    0.5s. If we get ~0.5s the tasks are GIL-bound serial and the entire
    24-week f3d1-volta plan dies on day 1.
    """

    def slow_passthrough(x: int) -> int:
        time.sleep(0.1)
        return x

    out, elapsed = run_parallel_timed_py(slow_passthrough, [0, 1, 2, 3, 4])
    assert out == [0, 1, 2, 3, 4]
    # Sequential would be 5 * 0.1 = 0.5s. Parallel should be close to 0.1s.
    # Allow up to 0.35s for tokio overhead + GIL acquire/release spikes
    # + Windows scheduling jitter (which is real).
    assert elapsed < 0.35, (
        f"elapsed {elapsed:.3f}s; tasks did not parallelize "
        f"(GIL-bound serial). The keystone plan dies if this is true."
    )


def test_no_callback_returns_empty() -> None:
    """Edge case: empty input list returns empty output."""
    out = run_parallel_py(lambda x: x, [])
    assert out == []


def test_returns_object_types_unchanged() -> None:
    """PyObject round-trips through tokio task without losing type."""
    inputs = ["hello", 42, [1, 2, 3], {"a": 1}, None, True]
    out = run_parallel_py(lambda x: x, inputs)
    assert out == inputs
