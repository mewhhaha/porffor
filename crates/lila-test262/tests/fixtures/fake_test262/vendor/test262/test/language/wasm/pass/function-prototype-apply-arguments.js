/*---
flags: [raw]
---*/

function pick() { return arguments[1]; }
pick.apply(null, [1, 2, 3]);
