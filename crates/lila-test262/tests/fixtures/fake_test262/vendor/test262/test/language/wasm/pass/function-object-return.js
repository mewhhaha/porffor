/*---
flags: [raw]
---*/

function box(x) {
  let o = { x: x };
  return o;
}

let o = box(2);
o.x;
