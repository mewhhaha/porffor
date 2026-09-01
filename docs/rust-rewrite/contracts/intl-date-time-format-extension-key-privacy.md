# Intl.DateTimeFormat extension-key privacy

Status: implemented as a source-equivalent T23 invariant closure.

The owner-private `IntlDtfRelevantExtensionKey` domain, with exactly the `Ca`,
`Hc`, and `Nu` variants, is the single authority for DateTimeFormat's relevant
Unicode extension keys. Its
private complete list drives lookup, option resolution, resolved-locale suffix
construction, and record storage. Other backend modules cannot construct a key
or introduce a competing iteration order.

The owner-private `IntlDtfExtensionResolution` exhaustively distinguishes the
canonical-string `ca`/`nu` record shape from the small-code `hc` shape. Each key
selects exactly one resolution, and the projections have no fallback arm.
Adding a key or resolution shape therefore requires every consumer to compile
exhaustively before the backend builds.

Restoring only the former visibility of both domains and the complete key list
reproduces the exact original 122-line source with SHA-256
`81c65d3e0cba0940b53421102caa0536bb0820d0acbe531d75abbeb8f555274e`.

Batch BL also makes the keyword-needle carrier owner-private. Its sole
constructor binds a relevant key to an accepted spelling, and the scanner can
consume only its borrowed bytes and length. Restoring only the former
visibility reproduces the exact original 22-line carrier with SHA-256
`ab8dfe30ef006fe674cf1ba6663072d923941c92b568bb8825a15c29ccb4b8a2`.
At the Batch BL checkpoint, `cargo xc` is green, the strengthened privacy
target passes `3/3`, and the exact calendar extension leaf passes both Wasm-AOT
executions with every failure bucket at zero.

This source-equivalent hardening has no new Intl behavior and does not close T23.
It changes no accepted extension, canonical spelling, option observation,
record slot, emitted instruction, Test262 materialization, or published count.

At the Batch BI checkpoint, `cargo xc` is green and the recursive privacy
target passes `3/3`. The exact calendar canonical-string and hour-cycle code
resolution leaves pass all four Wasm-AOT executions with every failure bucket
at zero.
