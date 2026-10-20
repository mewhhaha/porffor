/*---
flags: [raw]
---*/

let f = () => this;
f() === globalThis;
