function f(a, b) {
  if (a === b) {
    return a !== 0 || false;
  }
  return false;
}

if (!f(true, true)) throw "short-circuit";
