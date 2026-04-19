/*---
flags: [raw]
---*/

let p = { x: 1 };
let o = Object.create(p);
Object.getPrototypeOf(o) === p;
