/*---
flags: [raw]
---*/

function F() {
  this.x = 1;
  return { y: 2 };
}

let x = new F();
x.y;
