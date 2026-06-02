function f(a, b) {
  if (a === b) {
    return 1 / a === 1 / b;
  }
  return false;
}

if (!f(true, true)) throw "division";
