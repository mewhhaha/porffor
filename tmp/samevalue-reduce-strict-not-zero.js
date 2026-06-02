function f(a, b) {
  if (a === b) {
    return a !== 0;
  }
  return false;
}

if (!f(true, true)) throw "strict-not-zero";
