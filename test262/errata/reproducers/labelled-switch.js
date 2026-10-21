// CaseBlock initialization must also reach declarations in an unvisited case.
var before, after, outside;
switch (1) {
  case 1:
    before = f();
    break;
  case 2:
    label: function f() { return 42; }
}
switch (2) {
  case 1:
    break;
  case 2:
    label: function g() { return 42; }
    after = g();
}
try { outside = g(); } catch (error) { outside = error.name; }
print(JSON.stringify([before, after, outside]));
