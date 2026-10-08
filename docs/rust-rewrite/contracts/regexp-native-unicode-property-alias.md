# Native RegExp Unicode property alias authority

Status: source batch; compilation and execution remain unverified.

The native `RegExpProgram` parser validates a complete code-point property
expression before acquiring ranges. The private-field, non-Copy
`RegExpCodePointProperty` owns the exact spelling and one closed property kind:
binary, General_Category, Script, or Script_Extensions. The actual range
resolver and Unicode 17 delta consume that owner. Raw names cannot select ICU
binary properties or populate a missing ICU base at either consumer boundary.

The complete existing `regress` binary-property enum and strict alias parser
supply ECMAScript's binary domain. An exhaustive match projects every one of
its 53 variants to the pinned ICU marker or the existing Any, ASCII, and
Assigned definitions. Adding a provider variant requires an explicit range
consumer arm. ICU's broader `new_for_ecma262` name lookup is retired from the
native parser, so `Id_Start`, `Id_Continue`, `Ids_Binary_Operator`, and
`Ids_Trinary_Operator` are syntax errors. Their normative names and aliases
remain accepted: ID_Start/IDS, ID_Continue/IDC, IDS_Binary_Operator/IDSB, and
IDS_Trinary_Operator/IDST.

The constructor first requires a nonempty ASCII property-value word made only
of letters, decimal digits or underscore; provider-specific punctuation and
non-ASCII aliases cannot enter the domain. The existing strict ICU
GeneralCategoryGroup and Script parsers retain their complete exact value-alias
domains. Only General_Category/gc, Script/sc, and
Script_Extensions/scx select a named family. A lone value may denote a general
category or binary property; a lone Script value does not. The committed
Unicode 17 additions factor into a closed four-value Script owner with the
same eight exact aliases and unchanged ranges for Script and Script_Extensions.
Other delta rows still patch only a resolving base. No invalid family/value
combination can acquire ranges from those rows.

The existing UnicodeSets property-of-strings consumer remains separate and
uses the provider's seven exact names and sequences. String properties require
`v`; `\P` and negated classes reject their static MayContainStrings witness.
The current native operand folding, `u` complement-before-fold order, `v`
fold-before-complement order, range publication, scoped modifiers, and matcher
instructions remain unchanged. Legacy `\p` identity grammar also remains.

ECMAScript requires exact property and alias spellings and forbids other names;
property-family/value restrictions and the string-property early errors are
specified in [ECMA-262 2026 §22.2.1.1 and §22.2.2.9.7–8](https://tc39.es/ecma262/2026/multipage/text-processing.html#sec-patterns-static-semantics-early-errors).

The added finite native semantic controls cover all four rejected ICU spellings
at bare and class `p`/`P` sites, valid aliases and complements, Unicode 17
membership, exact family/value distinctions, malformed/punctuation/non-ASCII words,
historical GC aliases, all four new Script names in both families, string-property restrictions, and Legacy identity admission. Existing
fold/complement and finite string controls remain required at verification.
No compiler, test, runtime, validator, oracle, generator, or resource probe ran
for this source batch.

The follow-on [computed code-point property extension](runtime-regexp-codepoint-property.md)
consumes this same constructor through one complete cached catalog and immutable
per-module image. It introduces no host ABI or matcher representation. Positive
v string properties remain a separate explicit capability gap.
