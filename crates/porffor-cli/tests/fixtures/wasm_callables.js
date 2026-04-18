function inc(x) {
  return x + 1;
}

function getX() {
  return this.x;
}

function pick() {
  return inc;
}

let g = inc;
let o = { x: 3, f: getX, inc: inc };
let a = [inc];

g(2);
o.inc(2);
o["inc"](2);
a[0](2);
pick()(2);
o.f();
18;
