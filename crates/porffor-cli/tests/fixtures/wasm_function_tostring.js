function demo(x) { return x + 1; }
let arrow = y => y + 2;
let o = { m(z) { return z + 3; } };

demo.toString() === "function demo(x) { return x + 1; }"
  && arrow.toString() === "y => y + 2"
  && o.m.toString() === "m(z) { return z + 3; }";
