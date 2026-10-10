# UTF-16 literal property keys and runtime string values

An ECMAScript String is a sequence of UTF-16 code units. A literal property key
containing the lone unit `D800` must remain distinct from the six-character
backslash spelling `\uD800`. The interner's Display conversion escapes lone
units for diagnostics and cannot supply a semantic property key.

`ScriptLowerer::interned_runtime_string` reads the interner's UTF-16 units and
uses the existing lossless StringPool encoding. Object literal keys, class
public keys, ordinary and resumable object-pattern keys, literal static-key
facts, and inferred method names share this conversion. Literal and template
value producers use the same encoding, including doubling a literal occurrence
of the private encoding marker. Identifier binding names and the scalar host
module-catalog domain keep their separate conversions and representability
checks.

The backend's existing StringPool decoder publishes the original code-unit
arrays. No alternate string representation or runtime comparison rule is added.
The original embedded-module control keeps exact request/attribute matching,
including duplicate final-value behavior. New IR controls distinguish all three
key spellings in both emitted keys and shape facts. Native controls exercise
strict/sloppy object and class keys, method names, updates/deletion, ordinary
and suspended patterns, rest exclusions, and drained async-job output under
30,000-ms execution deadlines.

Verification results belong to [the cloud receipt](../cloud-continuation-20261009.md). Staging these
owners and controls alone is not a native or pinned Test262 PASS.
