# Obsolete lowering specialization removal

Status: implemented as a source-equivalent T02 reachability closure.

The IR lowerer contained disconnected generator-expression rewrites, a
String-valued static-generator fold, a private-brand projection and an
operand-shape projection. None had a product call site. The live generator
declaration/resumable path, numeric static-generator authorities, iterator
method lowering, String literal folding and heap-shape merge remain.

`GeneratedFunctionOutput` also carried `function_id` and `this_info` fields
that its class-definition consumers never read. Its public-to-the-crate state
now consists only of the two observed results: `return_info` and
`construct_this_info`. The disconnected String-generator fold was the sole
user of the broad crate-root `regress::Regex` import; regexp compilation keeps
its direct `regress` import in `regexp.rs`.

The deleted private-brand projection has SHA-256
`5fa129a28e54d16a8d17a6d160906b0c4e018205424be6173ed5571d2fadf9b2`.
The deleted 527-line generated-iterator island has SHA-256
`8ee9816ca0c120d3d1513ac8b831c3a0783f39b7db85b431c12ee89502a1c5a9`.
The deleted generator-body array projection has SHA-256
`02dbdf1e8f7aa05681dffa2ef505eade66622569ac53cde246e368a25ae737ff`.
The deleted delegated-yield projection has SHA-256
`1320eddae0b215dfd5cc7f4f36bdaae2b85aa0738dea59bdbd6a4835a6faf9d8`.
Its deleted non-object delegate-method probe has SHA-256
`092a89c3965593b028c642d690c41a3c5bce5089396c747c49cdbf30c3a7d518`.
The deleted 283-line String-generator fold has SHA-256
`55261d8d96ceb75dbbece9833835c68d4a56c695ad8f3bbce3288145cb6efeba`.
The deleted declaration-by-name root has SHA-256
`92e5b6db98afaf7bb5c97c1db79246f5b3d5ea40408b15b0f48d82d65c5958e3`.
The deleted operand-shape projection has SHA-256
`326e77a61a4c63276a206c7eb836621ba4b8bfb3f1e3bb44ce7ca914904abef6`.
The deleted String-generator loop domain has SHA-256
`02a744bb3487bffa56d2fc11df81f51d862a0ee57da8aa56c88af443e5465530`.
The removed output fields, their three writes and the broad Regex import have
combined SHA-256
`c78546c57688c1d6cbb796baf74584b5c4bc61c448ff6541212aed7efa88d974`.

This reachability closure has no new JavaScript behavior and changes no
lowered IR: every removed specialization lacked a product root, and every
removed field lacked a read. It adds no Test262 materialization, capability
claim or published count.

At the Batch BW checkpoint, `cargo xc` is green with no `lila-ir` or
`lila-aot-wasm` warnings, the focused absence target passes `3/3`, and the
retained generator-expression, class-field and regexp-literal units pass
`3/3`.
