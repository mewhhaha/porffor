// A label chain exercises LabelledItem : Statement in lexical collection.
var before, after, outside;
{
  try { before = f(); } catch (error) { before = error.name; }
  outer: inner: function f() { return 42; }
  try { after = f(); } catch (error) { after = error.name; }
}
try { outside = f(); } catch (error) { outside = error.name; }
print(JSON.stringify([before, after, outside]));
