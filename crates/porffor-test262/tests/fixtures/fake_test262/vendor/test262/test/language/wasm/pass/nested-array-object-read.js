/*---
flags: [raw]
---*/

function make() {
  let o = { items: [{ x: 1 }, { x: 3 }] };
  return o;
}

make().items[1].x;
