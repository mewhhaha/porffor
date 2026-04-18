/*---
flags: [raw]
---*/

function make() {
  return { get x() { return 5; } };
}

let o = make();
o.x;
