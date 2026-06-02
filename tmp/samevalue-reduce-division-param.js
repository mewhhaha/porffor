function f(a) {
  return 1 / a;
}

if (f(true) !== 1) throw "param-division";
