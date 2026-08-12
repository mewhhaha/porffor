/*---
flags: [raw]
---*/

let g = function f(x) { return x; }.bind(null, 1);
g.toString();
