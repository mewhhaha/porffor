/*---
flags: [raw]
---*/

function F() {}
function G() {}
let x = new F();

(x instanceof F) && !(x instanceof G);
