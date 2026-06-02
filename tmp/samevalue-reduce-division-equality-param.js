function f(a, b) {
  return 1 / a === 1 / b;
}

if (!f(true, true)) throw "param-division-equality";
