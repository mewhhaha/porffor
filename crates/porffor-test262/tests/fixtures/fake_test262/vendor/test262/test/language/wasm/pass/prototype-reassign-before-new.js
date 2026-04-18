/*---
flags: [raw]
---*/

function F() {}
F.prototype = { x: 7 };

let x = new F();
x.x;
