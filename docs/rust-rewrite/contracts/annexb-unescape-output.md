# Annex B `unescape` materializes one canonical ECMAScript String

Status: normative for the Wasm-AOT implementation of Annex B.2.1.2
`unescape`.

## Semantic boundary

Annex B.2.1.2 decodes each recognized `%XX` or `%uXXXX` escape to one UTF-16
code unit and otherwise copies the next input code unit. The result is the
concatenation of those code units. In particular,
`unescape("%uD801%uDC01")` and `"\u{10401}"` are the same ECMAScript String.

The Wasm-AOT payload uses canonical UTF-8 for a paired lead/trail surrogate
and WTF-8 only when a surrogate remains lone. String equality is a payload
byte comparison. Emitting every decoded `%uXXXX` independently therefore
does not preserve the semantic boundary: the pair above becomes two
three-byte WTF-8 sequences instead of the canonical four-byte scalar.

`unescape` must first decide the output UTF-16 unit stream, then materialize
that stream through one pairing coordinator. Escape recognition and storage
encoding are separate phases.

## Output lifecycle

The coordinator owns one private, non-`Copy` pending-lead local. Its only
operations are:

1. consume one decoded UTF-16 unit;
2. retain a lead surrogate until the following unit is known;
3. combine a retained lead plus a trail into one scalar and emit canonical
   four-byte UTF-8;
4. flush a retained lead as three-byte WTF-8 before a non-trail unit or at end
   of input;
5. emit every other unit immediately; and
6. consume the pending-lead witness to flush, calculate the completed byte
   length and pack the one result payload.

The parser cannot write decoded units directly to the destination. Raw input
scalars also pass through the coordinator: a scalar above U+FFFF is projected
to its two UTF-16 units before consumption. This makes pairing work across
all token boundaries, including `%u`/`%u`, `%u`/raw and raw/`%u`, without a
source-pattern special case.

The decoder cannot pack the output separately from finalization. The one
`finish_into_payload` operation consumes the non-`Copy` witness, emits any
pending lead before measuring the result and owns the final payload pack.

The zero value of the Wasm local represents no pending lead. That encoding is
valid because every lead surrogate is in `0xD800..=0xDBFF`; no pending state
can be confused with the U+0000 input unit.

## Evaluation and abrupt completion

The existing builtin entry point still performs `ToString` exactly once
before entering the decoder and returns immediately if conversion throws.
The output coordinator performs no JavaScript calls and cannot introduce a
new abrupt completion. Malformed escape spellings remain literal input; this
operation never throws for malformed `%` syntax.

## Durable evidence

The product fixture pins:

- a decoded lead/trail pair against both an astral literal and its UTF-16
  length/code units;
- lone lead and lone trail preservation;
- a non-pairing lead followed by an ordinary unit;
- pairing across decoded/raw and raw/decoded boundaries;
- malformed input with raw multibyte BMP and astral neighbors; and
- the existing malformed, coercion-order and representative BMP cases.

A Rust structural test pins the private coordinator, requires the unescape
decoder to route units through it, and forbids direct decoded-unit calls to
`emit_store_utf8_codepoint` or direct result packing inside the decoder body.
It also pins final flush, completed-length measurement and packing order inside
the consuming finalizer.

## Nonclaims and deferred gates

This seam does not change `escape`, URI encode/decode, RegExp escaping or the
general String concatenation representation. It does not close the complete
Annex B, global-function or T24 trees, and it changes no published conformance
count until the centralized current-pin Wasm-AOT run verifies the result.

The static freeze gates are exact-file `rustfmt --check`, fixture syntax
checking, focused source/lifecycle inspection and `git diff --check`. Cargo,
fixture execution, focused pinned `unescape` leaves and the broad Annex B/T24
verification ladder remain deferred until independent review and completion of
the active centralized matrix.
