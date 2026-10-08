# Number builtin policy domains

The actual native Number entries use a closed predicate domain and an
exhaustive prototype-operation domain. Predicates admit only Number primitives
and perform no coercion. Prototype operations first extract a Number primitive
or the primitive data of a Number box; brand failures precede argument hooks.
Formatting receives typed Number bits and a complete result owner.

The constructor distinguishes an absent argument from explicit undefined,
performs its primitive numeric conversion before prototype lookup, then either
returns the Number or publishes a complete PrimitiveBox with the resolved
NewTarget prototype and stored primitive. No header is published for later
prototype or primitive repair. See [Number](https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-number-constructor-number-value).

Locale formatting consumes the retained whole primitive through the intrinsic
NumberFormat seam. The NativeIntl and Operations formatter bodies are separate
owners in the same atomic source batch; no scalar/tag adapter is admitted.
Existing native Number CLI controls remain; new GC controls cover receiver
validation and coercion/prototype ordering. None were executed during authoring.
