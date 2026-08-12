/*---
flags: [raw]
---*/

function f() { return this instanceof String; }
f.apply("x", []);
