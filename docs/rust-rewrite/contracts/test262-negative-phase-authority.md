# Test262 negative-phase authority

`NegativeExpectation.phase: NegativePhase` is the sole admitted phase carried
by a discovered negative test. Its four values are `parse`, `early`,
`resolution`, and `runtime`; code after discovery cannot substitute an
interchangeable string.

The YAML-like frontmatter parser is the only free-text boundary. A missing
phase retains Test262's `NegativePhase::Runtime` default, but a present unknown
`negative.phase` rejects discovery with both the test path and offending
spelling. It is never silently reclassified as a runtime negative.

Phase spelling, compile-only routing, failure ownership, and diagnostic-phase
matching consume the enum through exhaustive matches. Adding a fifth phase
without defining each of those laws is a compile error. Likewise, constructing
a negative expectation with `"parse".to_string()` is a type error rather than
a latent routing bug.

The focused unit regressions admit all four canonical spellings and reject
`run-time` with path evidence. The standalone
`negative_phase_authority_structure` target pins the typed field, the fallible
discovery boundary, the absence of the former catch-all classifier, every
downstream typed route, and this contract's T26 evidence.
