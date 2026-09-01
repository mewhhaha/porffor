# Try-clause empty-completion seed

## Backend boundary

The Wasm backend carries the current completion in four locals:

| local | meaning |
|---|---|
| `result_local` | value payload |
| `result_tag_local` | value tag |
| `completion_local` | normal, throw, return, break, or continue kind |
| `completion_aux_local` | break/continue target or zero |

This layout has no `Empty` value and no value-presence bit. A catch or finally
clause nevertheless starts with an empty statement-list accumulator. The
backend represents that entry state by calling
`emit_statement_result(function, ValueKind::Undefined)`. The helper writes the
Undefined pair, payload zero plus the Undefined tag, then writes Normal and a
zero auxiliary value.

That seed must occur after the incoming completion has been consumed. A catch
first copies the thrown value into its parameter binding. A finally path first
saves or pushes the try completion that it may need to restore. Seeding before
either step would discard the value the clause is meant to observe or preserve.

## Twelve clause entries

The backend has three execution shapes for try statements: ordinary,
generator, and async. Each shape has a catch-only owner, a finally-only owner,
and a combined catch/finally owner.

| owner | catch seeds | finally seeds |
|---|---:|---:|
| `compile_try_catch` | 1 | 0 |
| `compile_generator_try_catch` | 1 | 0 |
| `compile_async_try_catch` | 1 | 0 |
| `compile_try_finally` | 0 | 1 |
| `compile_generator_try_finally` | 0 | 1 |
| `compile_async_try_finally` | 0 | 1 |
| `compile_try_catch_finally` | 1 | 1 |
| `compile_generator_try_catch_finally` | 1 | 1 |
| `compile_async_try_catch_finally` | 1 | 1 |
| **Total** | **6** | **6** |

The module-boundary check extracts these nine owner functions and requires one
seed in each catch-only or finally-only owner and two in each combined owner.
A missing or duplicate call fails the repository gate.

## Completion-value behavior

A first `break` or `continue` in a catch block must not retain the thrown value
that was in the result locals while the catch parameter was initialized. A
first `break` or `continue` in a finally block likewise must not retain the
saved try value. Those statements update the completion kind and target, but
they can leave the value pair untouched. The clause-entry seed makes that pair
Undefined instead of leaking state from outside the clause.

The seed does not erase values produced inside the clause. An expression
statement compiled before a later `break` or `continue` overwrites the result
locals. The later abrupt statement therefore carries that preceding expression
value through the backend's existing statement-list completion behavior.

## Typed completion gap

Undefined is an encoding substitute for an empty completion value, not a typed
model of one. `Completion` and `UpdateEmpty` remain
`TrackedGapReason::ModelWithoutCallSite` rows. Their callerless Rust model does
not become product evidence because these backend writes do not construct it.

Closing that gap requires the product completion path to carry an explicit
`CompletionValue::{Empty, Present}` domain. Only then can `UpdateEmpty` consume
the distinction without confusing an empty value with the JavaScript value
`undefined`.

## Nonclaims and verification

This repair does not redesign the four-local completion ABI, add a word that
records value presence, or promote the `Completion` and `UpdateEmpty` catalog
rows. Complete try/catch/finally, generator, and async conformance remain out of
scope. This checkpoint makes no Test262, published-status, or conformance-count
claim.

At the central checkpoint, `cargo check -p lila-aot-wasm` is green with only
the pre-existing vendored parser warning. The exact twelve-entry structure
target passes `2/2`. Four independently observed CLI programs prove the empty
and present-value cases for both catch and finally; the two Rust integration
tests pass `2/2`. The neighboring abrupt-finally and derived-return controls
pass `2/2`. Formatting, module-boundary, task-plan, exact-shortcut and diff
checks are green, with the shortcut inventory unchanged at 240 entries.
