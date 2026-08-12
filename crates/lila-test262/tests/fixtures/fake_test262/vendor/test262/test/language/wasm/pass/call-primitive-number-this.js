/*---
flags: [raw]
---*/

function f() { return this instanceof Number; }
f.call(1);
