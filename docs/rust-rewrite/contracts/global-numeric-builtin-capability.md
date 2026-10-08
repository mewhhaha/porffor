# Coercing global numeric builtins

The consumed closed `GlobalNumericBuiltin` domain selects `isFinite` or `isNaN`
through the existing native dispatcher. Each call reads a whole argument from
the callable's private GC vector, performs the shared whole `ToNumber`, and
reads binary64 bits only on Normal. A thrown object is copied intact through
native cleanup. Missing arguments become Undefined; already-evaluated ignored
arguments retain their caller effects.

The result is a whole Boolean value. No raw payload/tag locals, integer object
addresses or reimplementation of StringToNumber remain. The IR continues to
retain the invocation instead of folding object/array or string arguments with
an unproved shortcut. The predecessor spelling guard is retired.

This T05 source is uncompiled and unrun. Existing IR/Engine semantic controls
remain, with an additional GC control for original Throw identity, ignored
argument effects and numeric primitive distinctions. Historical ref77 type
checks apply only to their predecessor source. Task acceptance waits for the
later complete batch checkpoint under the confirmed 4096 MiB aggregate cap.
