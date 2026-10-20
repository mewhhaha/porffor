/*---
flags: [raw]
---*/

function F() {
  this.x = 3;
}

let x = new F();
x.x;
