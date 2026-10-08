# Native result Arrays use the sole indexed storage owner

Private RegExp result/capture/indices Arrays and Object enumerable-own-property
Arrays publish complete data descriptors through emit_array_indexed_publish_descriptor.
Private RegExp reads retain a nullable descriptor proof until the corresponding
completed producer requires a present entry. No native consumer reads or writes
a dense backing slot, truncates a property index to I32, or derives Array length
from indexed occupancy.

The Object result builder retains its private capacity/next invariant, publishes
the descriptor before advancing next, and finishes by writing actual result
length before exposing the Array. RegExp producers retain their defining-Realm
prototype and explicit result length. Existing whole values, accessor absence
and descriptor attributes retain their owners.

The shared sparse index authority owns lookup, complete publication, uniqueness
and actual-index validation. This source successor consumes its fixed actual
GC APIs; the complete central schema/Objects/registry cutover remains in progress.
Existing meaningful Object and RegExp Engine cohorts remain unchanged. No source
mirror controls are added. Current compilation, Wasm validation, runtime and
pinned tests are unrun until all tasks finish source implementation, followed by
the confirmed aggregate 4 GiB serial verification checkpoint.
