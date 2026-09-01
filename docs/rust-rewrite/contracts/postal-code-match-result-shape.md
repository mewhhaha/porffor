# Postal-code match result shape

Status: implemented for the specialized postal-code pattern used by global
`RegExp.prototype[Symbol.match]` and non-global execution.

## Boundary

The specialized matcher has one private
`builtins/string/postal_code_match_result_shape.rs` owner. Its non-copyable
`PostalCodeMatchResultShape::{GlobalMatchArray, ExecMatchArray}` domain and raw
shape-parameterized emitter are child-private. The String parent can request
only the global or exec semantic operation; it cannot name, construct, import
or project the raw shape. One direct exhaustive match selects the result Array
length: one element for the global match and three for the exec result. A
second direct exhaustive match keeps global publication empty and gives the
exec result both captures, an `undefined` absent optional capture, and the
UTF-16 `index` and original `input` properties. No Boolean, default or wildcard
policy remains.

Match discovery, no-match `null`, capture positions and the common full-match
element remain outside those projections and are unchanged.

## Durable evidence

`postal_code_match_result_shape_structure.rs` recursively pins the private
module and parent exclusion, the two variants and lack of convenience
capabilities, both exhaustive projections, the shared discovery prefix, the
complete exec-only publication, both result tags, each semantic wrapper's sole
parent caller and the private definition-plus-two-wrapper-call census.

The moved four-line domain and 357-line raw emitter retain SHA-256
`2c218b01e482cf283729f52db2c171b9dddd0d6fbe1d4eac5bf2fb79fdc0ac71`
and `06fe70a126949e33e1cba69b6f349cf83d960a8e9961eecf65bbf5fc33c540d8`;
their combined 361-line semantic selection retains
`46a993e3cd8087a333de80d918ecd59d8a80af99acf1da5edb63ed0af18b4668`.
Only the two narrow semantic wrappers are new. The resulting 398-line child
has SHA-256
`fc2d538c93855feb1e1f011af9d2851d42f9b6c8db6f59a15387ea93e89088b4`.

## Verification

The bounded structure target passes all `3/3` tests. The existing CLI fixture
passes `1/1`, and exact Test262 leaves `S15.5.4.10_A2_T6.js`,
`S15.5.4.10_A2_T7.js` and `S15.5.4.10_A2_T8.js` pass both variants (`6/6`)
with every failure bucket at zero. Workspace formatting and the diff check are
green. No broad Test262 run, semantic golden or published matrix status refresh
was needed; only the three exact README leaf results were refreshed.

Batch AI moved the complete raw owner source-equivalently and narrowed the two
parent calls without changing their argument order or emitted behavior. At the
shared checkpoint, `cargo xc` exits zero;
`postal_code_match_result_shape_structure`,
`string_literal_replacement_scope_structure` and
`global_ascii_class_quantifier_structure` each pass `3/3`, for `9/9` total.
The exact
`string::run_wasm_backend_succeeds_for_string_match_postal_code_fixture` CLI
witness passes `1/1`. Exact `S15.5.4.10_A2_T6.js`,
`S15.5.4.10_A2_T7.js` and `S15.5.4.10_A2_T8.js` pass sloppy and strict
execution (`6/6`) with every failure bucket at zero. No semantic golden was
needed or run. Final formatter, diff, module-boundary, task-plan and 240-entry
shortcut-inventory gates are green.

## Deferrals

This contract does not generalize RegExp parsing or execution, change other
static matcher shortcuts, or complete String, RegExp, T18 or T19.
