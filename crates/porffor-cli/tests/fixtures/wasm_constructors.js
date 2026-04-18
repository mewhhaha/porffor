function F() {
  this.x = 3;
}

F.prototype.marker = 1;

let a = new F();
a.x = 4;

function H() {}
H.prototype = { z: 7 };

function make(v) {
  return function K() {
    this.k = v;
  };
}

let K = make(5);
let k = new K();

a.x;
new H().z;
k.k;
19;
