let x = 1;

let o = {
  x,
  f() {
    return x;
  },
  get y() {
    return x;
  },
  set y(v) {
    x = v;
  },
  g() {
    return 0;
  }
};

o.y = 5;
o.f();
