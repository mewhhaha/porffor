/*---
flags: [raw]
---*/

function make() {
  return () => this.x;
}

let o = { x: 3, f: make };
let g = o.f();

g();
