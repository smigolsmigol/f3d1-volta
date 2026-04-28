"""f3d1-volta: the keystone of the f3d1 arch.

Phase 0 surface: validate that the pyo3+tokio architectural pattern works.
The real keystone API ships in Phase 2+.
"""

from f3d1_volta._f3d1_volta_core import run_parallel_py, run_parallel_timed_py  # type: ignore[attr-defined]

__version__ = "0.0.1"

__all__ = ["run_parallel_py", "run_parallel_timed_py"]
