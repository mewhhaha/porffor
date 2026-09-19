# String match call emission

The unchanged strict Test262 `built-ins/RegExp/named-groups/lookbehind.js`
passes on the frozen main baseline and exceeds the native compiler's function
size limit on checkpoint fifteen. Its main grows from 3,653,288 to 4,144,083
encoded bytes, with 104 declared locals in both. These artifacts were captured
from isolated native whole-program caches with the real harness preparation.

Each of its 22 ordinary `.match` calls used to emit three fallback branches.
A local-linked byte census identifies 66 fallback prefixes: from the argument
kind dispatch through RegExp creation and custom-hook dispatch, ending before
the default literal matcher. Each measured prefix grows from 13,585 to 20,485
bytes between the frozen compilers. Their combined 455,400-byte increase
accounts for most of the main's 490,795-byte growth. These are partial fallback
spans, not a size estimate for the entire operation. Array presence bookkeeping
is absent from the checkpoint-fifteen main and is not this regression's cause.

`String.prototype.match` already has a canonical callable body. An ordinary
`.match` call now evaluates its receiver, performs the existing runtime-kind
property read with the original receiver, evaluates every argument, then uses
the existing proxy-aware call dispatcher. The dynamic read boxes primitive
lookup targets without changing accessor `this`. Replacement and inherited
methods remain observable; a method captured by the read is not read again
after argument effects. The call supplies the original receiver and complete
argument vector. Each abrupt completion reaches the active catch/finally route.

When the property is the intrinsic, its existing body owns `@@match` lookup,
hook invocation, receiver conversion and fallback RegExp construction. The
existing method-reference planner retains that intrinsic and its dependencies;
no runtime helper, ABI, heap layout or grammar admission is added. Other String
methods and the intrinsic's existing fallback grammar remain separate scope.

`string_match_call_emission` validates real emitted Wasm and bounds incremental
caller growth for parameter-based matches and repeated named-lookbehind
literals. `aot_string_match_call` covers the unchanged pinned strict fixture
with pinned assertion sources, replacement/inherited methods, primitive
accessor receivers, symbol-hook order and identity, missing-hook coercion order,
catch/finally, Proxy get/apply and spread-argument evaluation. The original
Test262 single-case replay remains required in addition to this native target.

The staged repair has source and byte-evidence review only. The post-repair
native result and function sizes must be measured by the next frozen batch;
this note makes no post-repair size or conformance-count claim.
