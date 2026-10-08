# Selected Intl compilation profiles

`CompileOptions.intl_profile` selects `IntlCompilationProfile::Minimal` by
default, or `Custom(CustomProfileId)` for a validated named selection of the
complete pinned component domains. Custom currently changes the admitted
profile identity; it does not filter locale data. Conformance has no compilation
variant because its complete producer remains unfinished.

Preparation retains the chosen profile in the private `CompilationUnit`.
Ordinary emission, compiled-unit execution, cached source execution, current
thread routes and agent worker retries all consume that choice. The program
cache uses version-five framing: a closed profile discriminator and a
length-framed Custom ID join the existing source, goal, compiler, graph and
policy fields. Wasmtime module caches use the actual emitted bytes.

Compilation constructs one cheap selected-data owner per emission. Programs
that consume Intl data emit all twelve admitted frames and their canonical
identity, including code-only `Intl.supportedValuesOf` catalogue consumers.
Programs without Intl data keep lazy selection. Engine admission checks a
complete component graph and its exact identity while bytes are still inert,
before cache lookup, native compilation or module start. A hostless complete
bound graph is admitted; partial, duplicate, corrupt or unlabeled graphs are
rejected. Without host imports or components, unexpected identity metadata
retains its rejection.

The CLI accepts `--intl-profile minimal|custom:ID` for `build wasm` and Wasm
`run`, including Script and Module entry selection. Invalid IDs, duplicate
options and unavailable Conformance reject. Explicit selection on other
commands or backends rejects before reading source. Library Custom selections
reject C/native emission and spec-exec execution instead of losing the choice.

Source controls use ordinary Custom compiler and CLI producers. Library
controls compare all twelve frames and the complete canonical identity, run
the same compiled unit, then exercise cached Script/Module routes. Separate
code-only catalogue and locale-case controls exercise selected-data consumers.
The CLI control compares its actual dumped artifact against ordinary library
compilation and executes Script/Module entries. Existing cache and agent-policy
controls now include profile identity. The retained all-service Engine control
uses actual Custom compilation rather than replacing sections after emission.

This source batch has not run compilation or regressions. Acceptance requires
the combined all-types checkpoint, the retained and new focused controls, and
the affected sequential broad verification.
