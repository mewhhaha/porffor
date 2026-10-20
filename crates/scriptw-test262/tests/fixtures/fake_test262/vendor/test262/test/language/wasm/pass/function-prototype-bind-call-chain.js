/*---
flags: [raw]
---*/

function f() { return this.v; }
let g = f.bind({ v: 2 });
g.call({ v: 9 });
