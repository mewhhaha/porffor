// TopLevelVarScopedDeclarations is a separate, positive control.
var before = f();
outer: inner: function f() { return 42; }
print(JSON.stringify([before, f()]));
