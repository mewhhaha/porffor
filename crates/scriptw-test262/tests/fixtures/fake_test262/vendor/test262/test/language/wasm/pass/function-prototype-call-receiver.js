/*---
flags: [raw]
---*/

function f(x) { return this.v + x; }
let o = { v: 2 };
f.call(o, 3);
