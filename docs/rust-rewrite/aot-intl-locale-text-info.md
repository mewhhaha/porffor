# Intl.Locale text information

## Isolated source draft — 2026-10-02

This future source proposal implements `Intl.Locale.prototype.getTextInfo`
through Rust lowering, emitted Wasm and a pinned native direction provider.
Native and compiler source is joined against the reviewed labelled-region/week
predecessor. All fourteen final source checks pass, including both ten-test
Python profile-admission files. Independent review, compilation and runtime
verification remain pending. No full-suite status changes follow, and
T23 remains open.

The method checks its receiver's initialized Locale slot and reads the stored
canonical tag without observable receiver property access. Method arguments
are ignored. The emitted algorithm creates a fresh ordinary object in the
method's defining Realm, with a writable, enumerable and configurable
`direction` data property. This property exists even when its value is
`undefined`. Branding errors use the same defining Realm.

Direction uses an explicit canonical script before applying genuine likely
subtags. Pinned CLDR47 script metadata distinguishes RTL, LTR and unknown
directions. Both absent data and an unknown direction stay undefined;
`Brai`, `Zyyy`, `Zinh` and `Zzzz` are explicit unknown primary rows. Base language
and region feed likely-subtag selection when the script is absent. The `sd`,
`rg` and `fw` Unicode preferences do not override text direction.

Operation 40 extends the closed host catalog under ABI 14. Its native response
is `Option<LocaleTextDirection>` with two closed known directions. An exact
eight-byte little-endian word encodes unknown as `0`, LTR as `1` and RTL as `2`.
The Wasm response reader checks the exact written length and complete code
domain before transferring a consuming private response to the materializer.
The native host does not construct JavaScript objects or execute JavaScript.
Provider setup validates pinned data before exposing the typed operation, and
the composite provider identity binds its production kernel.

Four Engine controls run in both Script modes. They cover likely and explicit
scripts, constructor script overrides, unknown direction, ignored extension
preferences, private-use text, branding, poisoned receiver properties,
freshness, descriptors and cross-Realm behavior. They remain uncompiled.
The exact pinned `intl402/Locale/prototype/getTextInfo` scope has five physical
files and ten sloppy/strict executions, all awaiting a fresh source-bound CLI.

After final source checks and review, required focused execution includes:

```sh
cargo xc --offline --locked
cargo test --offline --locked -p lila-intl --lib provider::locale_text::
cargo test --offline --locked -p lila-intl --lib locale_text_wire::
cargo test --offline --locked -p lila-engine --test aot_intl_locale_text_info -- --test-threads=2
cargo test --offline --locked -p lila-aot-wasm --test intl_namespace_plan_structure
```

Fresh exact pinned execution and the broad workspace, fake-suite and whole
Intl/Temporal checkpoints remain required. Earlier source or runtime passes
cannot be inherited by this ABI 14 source.

Normative sources: [getTextInfo](https://tc39.es/ecma402/#sec-Intl.Locale.prototype.getTextInfo),
[TextDirectionOfLocale](https://tc39.es/ecma402/#sec-textdirectionoflocale),
[pinned CLDR47 script metadata](https://raw.githubusercontent.com/unicode-org/cldr/2ef784e3a4168bc2a43cd1b5b9839b6636f5899c/common/properties/scriptMetadata.txt).
