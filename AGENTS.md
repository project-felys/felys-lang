# Workspace structure
- `felys/` — interpreter library (zero external dependencies), crate root `felys/src/`
- `philia093/` — PEG parser generator CLI binary (`publish = false`, uses proc-macro2/quote/syn/clap)
- `grammar/` — `felys.peg` (language grammar) and `meta.peg` (generator's own grammar)

# `felys/src/` module layout
- `ast/` — source language AST node types
- `frontend/` — parsing output → desugar → CFG construction (stage I→II)
- `optimizer/` — SSA optimization → bytecode codegen (stage II→III)
- `runtime/` — VM execution + bytecode dump/load serialization
- `stdlib/` — built-in functions (nn tensor ops, io, etc.)
- `philia093/` — **auto-generated**, do not manually edit (see Codegen below)

# Codegen: regenerating parser code
The `philia093` binary appends a `philia093/` subdirectory to the target path, so the target must be the **parent** directory:
```bash
cargo run --bin philia093 grammar/meta.peg philia093/src/   # → philia093/src/philia093/
cargo run --bin philia093 grammar/felys.peg felys/src/       # → felys/src/philia093/
```

Each `.peg` file's `{ ... }` header block becomes the `use` imports in generated `core.rs` and `common/memoize.rs`. **If you move/rename a module referenced by generated code, update the grammar header AND regenerate.**

The generator writes the 2018-style module layout (no `mod.rs`): it overwrites the module declaration file `philia093.rs` (sibling of the generated directory), `core.rs`, `common.rs`, and all of `common/`. It does **not** overwrite `helper.rs` (hand-written, lives inside the generated directory).

# Testing
Integration tests in `felys/tests/` use an `exec()` helper (`tests/utils/mod.rs`) that compiles Felys source, then validates output across **4 optimization levels** (0, 1, 2, `usize::MAX`) and a **dump→load roundtrip** — 8 executions per test case.
```bash
cargo test                       # all tests
cargo test --test <demo|general|stdlib>   # single suite
```

# Standard commands
```bash
cargo check --workspace    # typecheck
cargo build --workspace    # build
cargo test                 # unit + integration
cargo clippy --workspace   # lint
cargo fmt                  # format
```

# Architecture
- Compiler pipeline: `PhiLia093::parse()` → `desugar()` → `codegen(opt_level)` → `dump()` → `III::load()` → `exec()`
- Public API re-exports: `BinOp`, `UnaOp` (from `ast`), `III` (from `optimizer::stage`), `Object` (from `runtime::object`), `PhiLia093`
- Edition 2024 — code uses let-chains and other 2024 features
