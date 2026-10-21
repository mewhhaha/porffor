// Separate lexical initialization from the optional outer-variable copy.
var before, after, outside;
{
  before = f();
  label: function f() { return 42; }
  after = f();
}
try { outside = f(); } catch (error) { outside = error.name; }
print(JSON.stringify([before, after, outside]));
