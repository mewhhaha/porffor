/*---
flags: [raw]
---*/

function add(x, y) { return x + y; }
add.apply(null, [1, 2]);
