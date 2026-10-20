// Construct this valid computed source through the runtime UnicodeSet compiler.
// Chained subtraction must preserve the source without throwing.
var parts = ["[a--b", "--c]"];
var source = parts[0] + parts[1];
var constructed = null;
var caught = false;
try {
  constructed = RegExp(source, "v");
} catch (error) {
  caught = true;
}
if (caught) throw "valid pattern became a JavaScript exception";
if (constructed.source !== source) throw "returned object changed source";
true;
