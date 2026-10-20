var hits = 0;
Object.defineProperty(Array.prototype, "65536", {
  set: function () { hits++; }, configurable: true
});
var body = "0,";
for (var i = 0; i < 16; i++) body = body + body;
var result = JSON.parse("[" + body + "-0]", {ignored: true});
var first = Object.getOwnPropertyDescriptor(result, "0");
var last = Object.getOwnPropertyDescriptor(result, "65536");
var prototype = Object.getPrototypeOf(result) === Array.prototype;
delete Array.prototype[65536];
print(result.length + ":" + result[0] + ":" + Object.is(result[65536], -0));
print(first.writable + ":" + first.enumerable + ":" + first.configurable);
print(last.writable + ":" + last.enumerable + ":" + last.configurable + ":" + prototype + ":" + hits);
var rejected = 0;
for (var text of ["[0,]", "[01]", "[1e]", "[1..0]", '{"a":}']) {
  try { JSON.parse(text); } catch (error) { if (error instanceof SyntaxError) rejected++; }
}
print("syntax:" + rejected);
