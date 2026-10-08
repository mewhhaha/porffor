# Date component operations and private GC DateValue

Status: source-authored in the atomic T05 GC cutover on 2026-10-04. Compilation,
Wasm validation, focused regressions and pinned conformance have not run for this
source. MAIN integration remains 117; historical passes do not verify this draft.

The actual Date allocator creates one `DateObject` with an ordinary object header
and a private F64 time field. A private branded capture reads this field before
any argument hook. A receiver must pass RefTest for the concrete Date record;
public marker properties, the Date prototype and Proxy wrappers cannot acquire
this brand. Only `CompletedDateClipLocals` reaches the time-field writer.

The closed `DateComponentGetter`, `DateComponentSetter` and `DateTimeBasis`
domains control actual dispatch. Calendar scalars are typed I64 locals containing
binary64 bits; object identity and semantic values remain rooted GC references.
Local full-year setters coerce year before projection. UTC full-year and Annex B
year setters prepare their captured coordinate first. Other setters coerce all
present arguments before the captured-NaN decision. A captured invalid Date
returns NaN without a field write, retaining mutations made by coercion hooks.

Full-year recovery uses local arithmetic zero directly. Local setters perform
MakeDate, compatible UTC selection and TimeClip; UTC setters skip zone projection.
Every ToNumber returns a whole completion and the first Throw branches to common
cleanup before any later conversion. The actual private writer retains the same
receiver identity through all hooks.

The obsolete source-spelling guard is retired. Existing Engine controls in
`aot_date_system_time_zone` exercise captured values, invalid no-write behavior,
recovery, conversion order, historical offsets and range-edge UTC selection.
These controls and the pinned suite are required later and remain unrun.
