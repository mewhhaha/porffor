// This valid computed source has no complete finite cache entry. The current
// runtime grammar gap must terminate outside JavaScript try/catch. A returned
// object would conceal the missing compiler semantics even without executing it.
var parts = ["[\\p{Lu", "}-]"];
var source = parts[0] + parts[1];
var constructed = null;
var caught = false;
try {
  constructed = RegExp(source, "u");
} catch (error) {
  caught = true;
}
if (caught) throw "valid pattern became a JavaScript exception";
if (constructed.source !== source) throw "returned object changed source";
true;
