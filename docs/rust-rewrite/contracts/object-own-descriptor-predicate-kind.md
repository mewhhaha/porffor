# Object own-descriptor predicate kind

The GC source draft keeps one private owner,
`builtins/object/own_descriptor_predicate.rs`, and a closed three-case policy:
ObjectHasOwn, PrototypeHasOwnProperty and PrototypePropertyIsEnumerable. Each
actual dispatcher entry constructs its matching policy. Exhaustive borrowed
matches select acquisition/coercion order and descriptor projection together.
The policy has no incidental Copy/Clone capability.

Object.hasOwn completes ToObject on its first argument before coercing the
second to a property key. Both prototype forms coerce their key before
ToObject(this), including the nullish receiver failure. All forms consume the
same completed typed own-descriptor acquisition. The first two project
presence; propertyIsEnumerable projects presence and the stored enumerable
flag. Original throws, Symbol keys and reference identities remain whole.

Historical Batch X/AK checks covered the preceding source. They do not establish
acceptance for this GC rewrite. The existing semantic predicate fixture remains
required. The obsolete source-spelling guard is retired. No compilation or
runtime control has run during dry authoring; final verification requires the
complete source pass and the confirmed 4096 MiB process tree cap.
