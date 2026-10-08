# Intl.ListFormat Wasm contract

The ListFormat shell emits the constructor, supportedLocalesOf, resolvedOptions,
format and formatToParts as real Wasm builtin bodies. JavaScript coercions,
property access, iteration, abrupt completion and allocation remain in Wasm;
the native service receives only an admitted locale, closed type/style choices
and exact UTF-16 strings. No public Intl method is used as a delegate.

The constructor reserves an ordinary object after GetPrototypeFromConstructor,
retaining both prototype payload and representation tag. That observation
precedes CanonicalizeLocaleList. GetOptionsObject accepts undefined or an
Object; localeMatcher is read before locale resolution, then type and style.
A reserved object becomes initialized only after those steps complete. The
sole brand guard grants the private record view used by methods and requests.
The prototype itself is ordinary and unbranded. Entry and created Realm
installation use the same constructor/property definitions. ListFormat has
one live prototype slot at byte 576, extending the Realm intrinsic record to
584 bytes; no unused slot is reserved.

supportedLocalesOf canonicalizes locales before CoerceOptionsToObject, so
primitive options are boxed in the called function's Realm. resolvedOptions
creates a fresh called-Realm ordinary object with locale, type and style in
that order. Parts arrays, their ordinary objects and intrinsic errors also
use the called function's Realm. A NewTarget primitive prototype falls back
to the NewTarget Realm's actual ListFormat prototype; Function, Array and
Proxy prototype values retain their tags and identity.

format and formatToParts require the real brand before reading their input.
Undefined supplies an empty list. Other inputs use the shared synchronous
GetIterator/IteratorStepValue owner: next is cached, done/value are live,
iterator-operation abrupt completions propagate without closing, and a
non-string value is rejected without coercion. Only that rejection calls
IteratorClose, once, preserving the original called-Realm TypeError against
return getter/call/result failures. An unpublished own array retains every
original string, including empty strings, duplicates and isolated surrogates.
Only successful completion constructs ObservedStringListLocals.

The shared intl_provider_wire owner serves NumberFormat, PluralRules and
ListFormat through eleven exhaustive request variants. A List formatting
request borrows both a branded record and its completed string list. The
request and response retain that list's lifetime; the decoder receives no
independent input count or separately paired list. Version/opcode/direction,
returned span, every word/count/unit sequence and exact final exhaustion are
checked. Original Element indices must be exactly 0..n-1 once in order.
Literal scopes must be nonempty and follow an Element; pinned nonempty final
suffix scopes are retained. Output selects original string payloads by those
validated indices. format concatenates the same partition that formatToParts
materializes. UTF-16 literal units use the existing WTF-8 encoder, preserving
isolated units and canonical surrogate pairs. Request capacity is checked
incrementally while measuring list fields and again for the complete message.
Malformed or resource responses fail at the private host boundary.

The exact UTF-16 input writer moved from the NumberFormat numeric child to
this shared owner; NumberFormat and ListFormat are its real callers. Existing
NumberFormat/PluralRules wire field order, numeric provenance, outcomes and
category-mask checks remain intact. Intl ABI 8 adds operations 21, 22 and 23;
the native provider contract owns the pinned data, nine effective template
choices and implementation-defined conditional string projection policy.

The normative shell authority is immutable ECMA-402 revision
`e463f3c8b62e5c67f4846cd05eec40d8d5947ed0`, captured in the ListFormat design
receipt. The service uses the pinned ICU4X source/data receipt, including
conditional Spanish/Hebrew patterns and nonempty suffix-bearing joiners.

This package has source/format/index checks only. The three lifecycle/wire
structure tests and Engine controls are authored and unexecuted. Required
runtime gates include the full 81-file ListFormat pinned cohort (162 modes),
adjacent NumberFormat/PluralRules controls, created-Realm and iterator behavior,
whole workspace and full Intl/Temporal suites. No support or conformance
percentage follows from the staged source checks.
