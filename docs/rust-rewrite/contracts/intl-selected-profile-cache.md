# Reusing admitted Intl selections

Repeated emissions of the same complete Intl compilation profile share one
immutable, constructor-admitted selected bundle. The process cache retains at
most one selection and at most 64 MiB of its actual encoded component frames.
This bound measures encoded data, not total provider heap usage. Active callers
own their bundles independently of cache eviction. Minimal keeps its existing
static selection.

The key is `IntlCompilationProfile` equality, including the Custom ID, public
services and every physical projection. A textual Custom name alone is never a
key. A miss still runs the original component producers and complete provider
admission. Failed builds do not enter the cache. An over-budget successful
selection remains usable by its caller without being retained globally.
Concurrent identical requests share the same admitted owner. There is no
persistent cache input or external mutable frame slot.

The selected bundle contains data and native provider proofs. Each JavaScript
execution still constructs its own Realm, Store, heap and intrinsic objects.
Supported-values memoization belongs to the same admitted provider identity;
cache reuse does not grant public operations from private dependency frames.
Export/import and runtime-cache admission retain their existing checks.

Focused native controls are authored for actual byte parity, export re-admission,
same-name service and locale separation, eviction, an over-budget selection,
failed-producer recovery and concurrent owner reuse. Native execution and the
measured before/after projection comparison are pending. The source seam avoids
repeated image construction; it does not yet establish a speedup.
