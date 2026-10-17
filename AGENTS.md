# Development Rules for nuop

## Commands

```bash
cargo build --manifest-path operator/Cargo.toml
cargo nextest run --manifest-path operator/Cargo.toml
cargo clippy --workspace --tests -- -D warnings   # warnings are errors
cargo fmt -- --check
```

## Testing Philosophy

### Test-Driven Development (TDD)

**Always follow the RED → GREEN → REFACTOR cycle:**

1. **RED** - Write a failing test first
2. **GREEN** - Write minimal code to make the test pass
3. **REFACTOR** - Clean up and improve the code

Never write production code without a failing test first.

### Test Organization

**No inline tests** - Tests must be in separate files in a top-level `test/` directory per crate:

```
operator/
  src/nuop/
    manager/
    reconciler/
  test/nuop/
    manager/
    reconciler/
```

- All test files live in `test/` at the crate root, mirroring the `src/` module hierarchy
- Test files in `src/` are forbidden — no `*_test.rs` or `test.rs` file may sit next to production code
- Use the module-aware naming convention:
  - Single-file module: `src/foo.rs` with `test/foo.rs`
  - Multi-file module: `src/foo/mod.rs` with `test/foo/test.rs`
  - Forbidden: mixed `src/foo.rs` + `test/foo/test.rs`
- The `mod` declaration name is independent of the filename — `mod foo_test` can point to `test/foo.rs`
- Test support files (`helpers.rs`, `support.rs`, `shared.rs`, `driver.rs`, `utils.rs`) also live in `test/`
- Keep test files focused and organized by module
- **Test file size limit:** No test file shall exceed 500 lines. When a test file reaches this limit, split it into topical subfiles in a subdirectory of the same name (e.g. `test/nuop/reconciler/controller.rs` → `test/nuop/reconciler/controller/reconcile.rs`, `test/nuop/reconciler/controller/scenarios.rs`, etc.). Shared support code stays in a `test/nuop/reconciler/controller/support.rs` file. The `#[path]` wiring in the source module declares each split file as a separate `#[cfg(test)]` module
- Declare each test module from its production module with `#[cfg(test)]` and a `#[path]` attribute. `#[path]` resolves relative to the declaring file, so the path must escape `src/`:
  ```rust
  // region:    --- Tests
  #[cfg(test)]
  #[path = "../test/foo.rs"]
  mod foo_test;
  // endregion: --- Tests
  ```
- From a subdirectory module (`src/foo/mod.rs`), add one more `../`:
  ```rust
  // region:    --- Tests
  #[cfg(test)]
  #[path = "../../test/foo/test.rs"]
  mod test;
  // endregion: --- Tests
  ```
- Do not write `#[path = "test/foo.rs"]` from a file in `src/` — that resolves to `src/test/foo.rs`, not the crate-root `test/` directory
- When unit tests in a source file become large, split them into topical files under `test/` and declare each one

### Test Categorization

nuop has a single crate and no `integration` cargo feature. All tests are unit tests.

**Unit test speed limit:** Every unit test MUST complete in under 1 second. A test that constructs a real Kubernetes client, binds TCP ports, or performs network I/O is an integration test — gate it behind a cargo feature before adding it.

**Mock the Kubernetes API.** Use `tower_test::mock::pair` to build a mock `kube::Client` and assert on the requests the operator issues. See `operator/test/nuop/reconciler/controller/support.rs`.

**Skip script-execution tests when `nu` is unavailable.** Tests that shell out to the Nushell interpreter call `should_skip_script_tests()` and return early when `nu` is not on `PATH`.

`cargo nextest` is the test runner. Profiles live in `operator/.config/nextest.toml`:

| Profile | Purpose |
|---------|---------|
| `default` | Unit tests only — fast feedback, high parallelism |
| `integration` | Reserved for future integration tests — unused (nuop has no `integration` feature) |
| `ci` | Full suite for CI — no retries, JUnit XML output |

Commands:

```bash
cargo nextest run --manifest-path operator/Cargo.toml                 # unit tests (fast)
cargo nextest run --profile ci --manifest-path operator/Cargo.toml   # CI run, JUnit output
```

In the Nix dev shell these are wrapped as `op-tests` (default profile) and `op-tests-ci` (CI profile, JUnit output).

The pre-push hook runs the full suite: `cargo clippy --manifest-path operator/Cargo.toml --tests -- -D warnings` lints all test code, then `cargo nextest run --profile ci --manifest-path operator/Cargo.toml` runs everything and writes `junit.xml`.

### No test-only code in production

**`#[cfg(test)]` on `fn` in production files is banned.**

- ❌ NO: `#[cfg(test)]` on any `fn` in production code (`.rs` files that are not in the `test/` directory)
- ✅ YES: `#[cfg(test)]` on `mod` declarations for test modules (e.g. `#[cfg(test)] mod foo_test;`)
- ❌ NO: Test-only accessor methods that expose private fields solely for tests
- ✅ YES: Design public API so tests use the same methods as production code
- ✅ YES: Test through behavior — if the public API doesn't expose enough to verify a behavior, the API is missing a method, not the test missing a backdoor
- ❌ NO: Making a field `pub(crate)` just so tests can peek at internal state — this is the same code smell as `#[cfg(test)]` accessors

**If a test needs to inspect private state, one of these is wrong:**
1. The test is testing implementation details — rewrite it to assert outcomes through the public API
2. The design is wrong — the internal state shouldn't matter, only its observable effects

**Pre-existing violations are still violations.** If you encounter `#[cfg(test)]` accessor methods while working on an unrelated task, do NOT fix them inline — create a separate task for the cleanup. But they ARE a code smell that must be addressed.

### Test Through the Public Boundary

- Tests verify behavior through the actual public API (e.g., `reconcile()` for the reconciler or `CommandExecutor::execute()` for script execution), not through internal types or methods exposed solely for testing
- No private helper functions made public just for tests — shared logic is a private implementation detail, tested indirectly through observable output
- If a test needs to inspect internal state, the test is testing implementation details — rewrite it to assert outcomes through the public API, not add a backdoor
- `pub(crate)` on a field or method just so tests can peek at internal state is the same code smell as `#[cfg(test)]` accessors

### Banned Test Anti-Patterns

These test patterns are **banned**. They verify what the compiler already enforces for free. They give false confidence by inflating the test count without testing any behavior.

- ❌ NO: **Compile-time trait-bound assertion tests** — `fn assert_send<T: Send>() {}` / `assert_send::<MyType>()`. If the type were not `Send`, the crate would not compile. The test adds zero information.
- ❌ NO: **`assert_sync` / `assert_sink` / `assert_<Trait>` probe tests** — same tautology. The compiler enforces trait conformance at every call site that uses the bound.
- ❌ NO: **Tests that only assert a type implements a trait** without calling a method and checking the result. Constructing a value and passing it to a generic function with a trait bound is testing the compiler, not the code.
- ❌ NO: **Tests with empty or trivial bodies** — every test must execute at least one function call and assert at least one observable result. A test that only checks a constant or a type-level property is not a test.
- ❌ NO: **Tests that re-verify compiler guarantees** — `Send`, `Sync`, `Unpin`, trait bounds, `Clone`, `Default` — these are all compiler-enforced. A test that asserts them adds zero value.

**What to write instead:**
- ✅ YES: Call the function under test with real arguments and assert on the return value or observable side effect
- ✅ YES: Use DI mocks (trait implementations in the test file) to inject controlled inputs and capture outputs
- ✅ YES: Every test must be behavioral — if the implementation is wrong, the test fails; if the implementation is right, the test passes; if the test passes regardless of the implementation, it is a bad test

### Manual Verification

When a task requires manual verification (e.g., running the operator binary against a cluster, observing controller behavior),
run the operator binary and verify by hand. Do NOT:

- Write nushell scripts that spawn the operator and poll its output to automate cluster interaction
- Write shell scripts that spawn the binary in a PTY and poll for exit
- Invent test automation that is not in the task spec

Manual verification means you run the command, observe the result, and report it.
If a behavior cannot be verified manually, flag it — do not script around it.

### Task Scope Discipline

Do exactly what the task spec says. If the task says "fix code", fix the code.
If the task says "write tests", write tests. If the task says "manual verification",
verify manually. Do not invent additional work, scripts, or tooling that is not
in the task's INCLUDED scope. If you discover work that needs doing but is not
in the scope, flag it for the orchestrator — do not do it inline.

### Dependency Injection for Testability

**Every external dependency MUST be injected via a trait so unit tests can substitute a mock.** This is not a suggestion — it is a hard requirement derived from the SOLID Dependency Inversion Principle.

- ❌ NO: A constructor or method that hardcodes a concrete function performing I/O (e.g., a constructor that opens a file or builds an HTTP client directly)
- ✅ YES: Define a trait (`CommandExecutor`) and store it on the struct. Production uses the real impl (`ProcessExecutor`); tests use a mock impl.
- ❌ NO: A unit test that constructs a real `kube::Client` against a live cluster, performs TLS init, or shells out to the `nu` interpreter
- ✅ YES: Inject a mock that returns a pre-built value. The mock lives in the test file, not in production code.
- ❌ NO: `pub(crate)` fields or `#[cfg(test)]` accessors to expose internal state for tests
- ✅ YES: Design the public API so tests use the same methods as production code. If the API doesn't expose enough to verify a behavior, the API is missing a method.

**The DI boundary is where the trait object lives.** nuop uses static dispatch — `State<E: CommandExecutor>` is generic over the executor, so no trait object is needed. If a crate boundary ever requires dynamic dispatch, that boundary is the only place it is allowed.

### What counts as production code

Any `.rs` file that is NOT in the `test/` directory is production code. This includes `mod.rs`, `controller.rs`, `state.rs`, `reconciler.rs`, `config.rs`, etc. The `#[cfg(test)]` attribute on `fn` in these files is the violation.

### Mocking

**Use mocks wherever available:**

**Mocking is mandatory, not optional.** See "Dependency Injection for Testability" above for the hard requirement.

- Mock external dependencies (the Kubernetes API, file system, network, HTTP clients, databases)
- Mock the Kubernetes API with `tower_test::mock::pair` when testing the operator's API calls
- Use dependency injection to make code testable — every external dependency gets a trait
- Trait-based abstractions for mockable interfaces — the mock lives in the test file, not in production code
- A unit test that constructs a real `kube::Client` against a live cluster, opens a real database, or shells out to the `nu` interpreter is an integration test in disguise — fix the DI, do not gate the test

### Code Coverage

Code coverage is measured with `cargo-llvm-cov` (LLVM source-based instrumentation, stable Rust).

```bash
cargo install cargo-llvm-cov --locked
rustup component add llvm-tools-preview
cargo llvm-cov --workspace --html --output-dir coverage/
```

- Coverage is a second quality dimension next to clippy: clippy finds suspicious code, coverage finds code no test executes.
- Test files live in `test/` (singular), which the default exclusion patterns do not cover — pass `--ignore-filename-regex '(/test/|/test$)'` to keep them out of the report.
- `--fail-under-lines <N>` exits non-zero when total line coverage drops below `N` percent. No threshold is enforced yet.
- Full workflow, output formats, per-crate runs, and CI integration: `docs/development/coverage.md`.

## Code Quality

- Write tests before implementation
- Keep functions small and focused
- Use meaningful names for tests (describe what they verify)
- Each test should verify one behavior
- Refactor only when tests are green
- No hidden global mutable state — use explicit state via structs

### Constructor Patterns

Follow the rust10x fluid API conventions:

- `Default` for sync empty constructors — do not use `new() -> Self` with empty arguments
- `new(...)` for the primary constructor with obvious common arguments
- `from_...(...)` for secondary constructors
- `with_...(self, ...) -> Self` for fluent chainable setters (consuming pattern, not `&mut self`)
- `append_...(self, ...)` for adding single items, `append_...s(self, ...)` for `IntoIterator`
- `set_...(&mut self, ...)` for property setters taking `&mut self`
- Use `impl Into<...>` for constructor and setter parameters where appropriate
- Separate `Builder` struct with `.build()` only when construction is complex, requires validation, or is fallible (returning `Result`)

### Code Structure (rust10x regions)

Organize source files in this order:

1. Primary public types
2. Supporting public types
3. Public functions
4. Public inherent implementations (grouped by purpose: Constructors, Chainable setters, Accessors)
5. Trait implementations (in separate `impl` blocks)
6. Private support functions/types (`// region: --- Support`)
7. Unit tests (`// region: --- Tests`)

Use code regions for meaningful sections:

```rust
// region:    --- Types
// endregion: --- Types

// region:    --- Froms
// endregion: --- Froms

// region:    --- Support
// endregion: --- Support

// region:    --- Tests
// endregion: --- Tests
```

Group `From` implementations together in a `Froms` region. Place private helpers in a `Support` region. Do not create regions merely to label every `impl` block.

### Error Handling

- Use `thiserror` for error enums (purpose-built, ergonomic `#[error("...")]` syntax)
- Define `pub type Result<T> = core::result::Result<T, Error>` in `error.rs`, re-export from `lib.rs`/`main.rs`
- `Custom` variant with `#[from(String, &String, &str)]` for easy `.ok_or("message")?`
- External error types below `// -- Externals` comment
- Application-specific variants above
- `Custom` variant always on top

### Modern Rust (Edition 2024)

- Use if-let chains: `if let Some(x) = foo && x > 0 { ... }` — do NOT nest `if` inside `if`
- Avoid `ref` bindings — use match ergonomics
- Inline format args: `println!("{name}")` not `println!("{}", name)` for simple variables
- `async || {}` closures supported (use `AsyncFn`, `AsyncFnMut`, `AsyncFnOnce` traits)
- Avoid manual pattern match when possible: `line.trim_start_matches([' ', '\t'])` not `|c: char| c == ' ' || c == '\t'`

### No Parallel Developer Agents

**NEVER run two developer agents concurrently on the same repository.**

Compiled projects share a build cache, lock files, and the working tree.
Parallel agents will corrupt each other's builds and produce interleaved file edits.

- Delegate one developer task at a time
- Wait for it to complete before delegating the next
- Researcher and reviewer agents may run in parallel with each other, but never alongside a developer

### Review Before Commit

**Always run a review before `git commit`. No exceptions.**

```bash
cargo clippy --workspace --tests -- -D warnings
cargo nextest run --manifest-path operator/Cargo.toml
```

A reviewer subagent must sign off before committing. Clippy warnings are
build failures. `#[allow(...)]` is never acceptable — fix the code instead.

### `unwrap()` / `expect()` Policy

- ❌ NO: `unwrap()` or `expect("...")` in production code
- ❌ NO: `unwrap()` or `expect("...")` in test or example code
- ✅ YES: `unwrap_or`, `unwrap_or_else`, `if let`, `?`, `.ok_or("should have ...")?`
- ✅ YES: `mutex.lock().unwrap()` — mutex poison is a fatal internal
  inconsistency and panicking is correct (the ONLY exception)
- For tests, use `.ok_or("should be ...")?` with `type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>`

### Nested `if` Statements

Flatten nested conditions. Never nest `if` inside `if` when they can be combined.

```rust
// ❌ WRONG
if let Some(x) = foo {
    if x > 0 {
        do_thing();
    }
}

// ✅ CORRECT
if let Some(x) = foo && x > 0 {
    do_thing();
}

// ✅ ALSO CORRECT — early return
let Some(x) = foo else { return; };
if x > 0 {
    do_thing();
}
```

### Module Files

Use `mod.rs` primarily for module declarations and intentional reexports. Put those declarations and reexports in a `Modules` region after any module-level documentation:

```rust
//! Module-level documentation.

// region:    --- Modules

mod event_base;
mod support;

pub use event_base::*;

// endregion: --- Modules
```

Implementation details should normally live in dedicated source files rather than in `mod.rs`.

### Scope Discipline

Do NOT refactor adjacent code "while you're at it." Every change must be
scoped to the task at hand. If something else needs fixing, create a new task.

### `#[allow(...)]` Is Never a Fix

Suppressing a clippy warning is not fixing it. If clippy flags something,
fix the underlying code. The only exception is `#[allow(dead_code)]` on
test helpers that are intentionally unused — and even then, prefer deleting
the dead code.

## SOLID Principles

**No dynamic dispatch! Use static dispatch with generics.**

- ❌ NO: `Box<dyn Trait>`, `&dyn Trait`, trait objects in internal code
- ✅ YES: Generics with trait bounds `T: Trait`
- ✅ EXCEPTION: crate-level DI boundaries can use dynamic dispatch (required for testability)
- Follow SOLID principles throughout:
  - **S**ingle Responsibility: One reason to change
  - **O**pen/Closed: Open for extension, closed for modification
  - **L**iskov Substitution: Subtypes must be substitutable
  - **I**nterface Segregation: Many specific interfaces over one general
  - **D**ependency Inversion: Depend on abstractions, not concretions

**DIP is a testability requirement, not just an architecture diagram concern.** If a constructor hardcodes a concrete function that performs I/O (kube client construction, TLS init, script execution, database opening), unit tests cannot substitute a mock. The test either becomes an integration test (slow, fragile) or the production code gets `#[cfg(test)]` backdoors (banned). The correct response is: define a trait, inject it, and let tests provide a mock impl. See "Dependency Injection for Testability" above.

### Trait Design

Every required trait method must be callable by at least one production code
path. If a method only exists to satisfy the compiler, the trait is designed
wrong — make it optional (provide a default implementation) or redesign the
interface.

```rust
// ❌ WRONG — run() is required but never called in production
pub trait Executor {
    fn run(&mut self, ...) -> Output;              // required, never called
    fn run_with_context(&mut self, ...) -> Output { // optional, always called
        self.run(...)
    }
}

// ✅ CORRECT — single method, covers all call sites
pub trait Executor {
    fn run(&mut self, ..., context: Option<&Context>) -> Output;
}
```

### No Delegation Chains

A delegation chain is a type that implements a trait by forwarding every method to
an inner field — `self.inner.method()` for each trait method. This is a SOLID
violation: the wrapper type has no responsibility of its own, it just adds a layer.

Delegation chains indicate the trait is implemented on the wrong type. If type B
owns the data and type A wraps B just to implement a trait, either:
1. Implement the trait on B directly (B owns the data, B owns the behavior)
2. Or eliminate the trait — if the caller can call B directly, the trait and the
   wrapper are unnecessary indirection

- ❌ NO: `Wrapper` implements `Renderer` by calling `self.inner.set_*()`
       which calls `self.state.set_*()` — 3 layers, zero behavior added
- ✅ YES: The caller invokes the owning type's method directly — no wrapper

If you find yourself writing `fn method(&mut self) { self.inner.method() }`, stop.
Either implement the trait on the inner type or eliminate the indirection.

## Configuration

nuop reads configuration from environment variables in `operator/src/nuop/config.rs`:

- `NUOP_SOURCES_PATH` — directory containing operator scripts (default `/sources`)
- `NUOP_MAPPINGS_PATH` — directory containing mapping YAML files (default `/config/mappings`)

Each operator script declares its own controller config (name, group, version, kind, finalizer, namespace, label/field selectors, requeue intervals) via `get_script_config()` in `operator/src/nuop/reconciler/util.rs`. Managed mode overrides these fields from mapping YAML files.

### Test Structure (rust10x conventions)

Every test function follows this layout:

```rust
#[test]
fn test_module_function_variant() -> Result<()> {
    // -- Setup & Fixtures
    // ... code that preps/sets the context for the test

    // -- Exec
    // ... code that executes the function to be tested

    // -- Check
    // ... assertions and verification
    Ok(())
}
```

- `type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>` at the top of every test module
- Test naming: `test_[module_path_name]_[function_name]_[variant]`
- Section comments in every test: `// -- Setup & Fixtures`, `// -- Exec`, `// -- Check`
- Use `// -- Exec & Check` when exec and checks are in a `for` loop
- No `unwrap()` — use `.ok_or("should be ...")?`
- Test support functions at the end under `// -- Test Support` (inline) or `// region: --- Test Support` (dedicated file)
- For temp data files, use paths like `tests-data/.tmp/test_function_name/`
