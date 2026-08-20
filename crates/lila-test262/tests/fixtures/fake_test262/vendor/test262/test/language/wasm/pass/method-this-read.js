/*---
flags: [raw]
---*/

function getX() {
  return this.x;
}

let o = { x: 3, f: getX };
o.f();
