/*---
flags: [raw]
---*/

function F() {}
F.prototype.getX = function () {
  return this.x;
};

let x = new F();
x.x = 4;
x.getX();
