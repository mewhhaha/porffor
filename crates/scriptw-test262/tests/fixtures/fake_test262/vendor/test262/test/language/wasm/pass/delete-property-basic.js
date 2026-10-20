/*---
flags: [raw]
---*/

let o = { x: 1 };
delete o.x;
"x" in o;
