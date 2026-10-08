# Native AbstractModuleSource intrinsic

The actual pin contains eight `built-ins/AbstractModuleSource` cases. They
observe the abstract constructor, its prototype, descriptors and tag getter;
none supplies a positive non-JavaScript module type. The source-phase staging
case requires SyntaxError for a loaded JavaScript Source Text Module, while the
language fixtures require host errors for missing targets. Those existing
loading, linking and dynamic source rejection owners remain authoritative.

The [Source Phase Imports draft](https://tc39.es/proposal-source-phase-imports/)
defines a Realm intrinsic with no global name. Lila now represents its
constructor with the existing native FunctionObject/FunctionContext and its
ordinary prototype with the same semantic GC object model. Two closed Realm
intrinsic slots cache those identities. Real bootstrap installs the native
constructor, complete common descriptors and the native tag getter before
publishing a Realm. The typed Standard catalogue records both bodies, their
native names, zero lengths and the constructor's unconditional throw.

The Test262-only `__lilaGetAbstractModuleSource` capability retrieves that
cached constructor from its own function's defining Realm. The harness no
longer substitutes a JavaScript class. Created Realm hooks and their returned
facade retain that Realm's own constructor. Ordinary Call and Construct throw
through the existing defining-function-Realm TypeError owner, without reading
mutable global TypeError or a NewTarget prototype property.

The supported host loads JavaScript Source Text Modules and genuine JSON
synthetic records. Both have empty `[[ModuleSource]]`, and no actual
module-source object can be created.
Accordingly the native tag getter returns undefined for every supported value,
including ordinary objects, inherited prototypes and revoked Proxies, without
reading their properties. No concrete foreign module kind, source object,
module execution adapter or interpreter bridge is invented. An optional future
loader with a source representation must supply its complete concrete record and source
representation through the same GC object model before widening this getter.

Three actual strict/sloppy Wasm cohorts cover native reflection and complete
descriptors, cache/GC identity, unconditional call and construction errors,
NewTarget property precedence, global poisoning and distinct created-Realm
identities. The current pin's eight cases and existing Source Text rejection
cohorts remain required verification. The original packet was source-only: no
Cargo, compilation, tests, runtime, exports or guards ran in that packet.
Positive loading of a concrete module-source object is an optional host extension;
the pin contains no such loader fixture. Full T12 acceptance and verification of
the joined source remain open.
