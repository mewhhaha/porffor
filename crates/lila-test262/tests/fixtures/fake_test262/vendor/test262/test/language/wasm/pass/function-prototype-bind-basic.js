/*---
flags: [raw]
---*/

function add(x, y) { return x + y; }
let inc = add.bind(null, 1);
inc(2);
