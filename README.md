# f3d1-volta

The keystone of the f3d1 arch.

A Rust-native concurrent agent execution runtime that closes the f3d1 ecosystem (f3dx, f3dx-cache, f3dx-router, tracewright, pydantic-cal, f3dx-bench) into one composable runtime where every existing piece is a node primitive. Drop-in for `langgraph.StateGraph`. tokio-parallel, lazy-compiled, replay-native.

> "LangGraph is pandas. f3d1-volta is polars."

## Status

**Phase 0 (architectural spike)**. Not for use yet. Validates the pyo3 + tokio + Python-callback-from-Rust pattern that the keystone depends on. Real API ships in Phase 2+.

See `docs/research/f3d1_volta_key_proposal_2026-04-29.md` (in the parent f3d1-pydantic-challenge repo) for the build plan.

## License

MIT.
