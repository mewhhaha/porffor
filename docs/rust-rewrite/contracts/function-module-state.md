# Function module state authority

## Closed construction role

Every emitted function has one private Rust construction role:

- `FunctionModuleState::Main(&FinalizedModuleGlobals)` borrows the sealed global
  package that the exported main body initializes and clears; or
- `FunctionModuleState::Internal` carries no module-global authority.

The shared `FunctionBuilder::new` owns this choice and moves it into the
builder. The main constructor is the sole `Main` producer. User functions,
host builtins, runtime-operation helpers and standard builtins are the four
exact `Internal` producers.

`FunctionModuleState` derives no cloning, copying, debugging, equality or
default-construction capability. Its four observations borrow the same value
and match both roles exhaustively:

1. the construction role selects the function return ABI;
2. main alone collects the main-frame cache bindings;
3. main alone initializes the runtime GC anchor root; and
4. main alone verifies and clears that root on a real main exit.

Adding a construction role therefore requires an explicit choice at every
projection. The construction-role value cannot be duplicated incidentally;
the existing private assembly boundary remains responsible for pairing the
sealed global package with its main body.

## Source-equivalent boundary

This is Rust-time compiler state only. The borrowed matches retain the existing
branches and their order. They add no Wasm ABI word, local, instruction, import,
type, global or section and are expected to leave emitted Wasm byte-identical.

The bounded recursive structure target pins the private attribute-free domain,
the 15-mention source census, five producer mappings, owned parameter and field
move, and the exact four borrowed exhaustive projections. The existing GC-root
unit witnesses decode the resulting module and prove that the root follows the
actual global inventory, is established before calls and is verified and
cleared on main exit:

```sh
cargo test -p lila-aot-wasm --test function_module_state_structure
cargo test -p lila-aot-wasm --lib tests::runtime_gc_root_follows_the_actual_fixed_and_template_globals -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib tests::runtime_gc_anchor_is_rooted_across_main_and_cleared_on_exit -- --exact --test-threads=1
cargo fmt --all -- --check
git diff --check
```

The structure target passes `3/3`. Each exact GC-root unit witness passes
`1/1`. Independent dry review is clean. The shared `cargo fmt --all --
--check`, `cargo xc`, diff, module-boundary and task-plan checkpoint is green
with the workspace's existing warnings.

## Non-claims

This capability closure does not change the Wasm GC schema, root layout,
collector policy, weak reachability, function ABI, main checkpoint behavior,
module assembly, heap representation or runtime capability checks. Broad and
golden verification remain batch-level work.
