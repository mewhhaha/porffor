var x = 1;

function f() {
  return this;
}

let g = () => this;

this === globalThis;
f() === globalThis;
globalThis.x;
g() === globalThis;
