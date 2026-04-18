/*---
flags: [raw]
---*/

let o = { x: 3, f() { return this.x; } };
o.f();
