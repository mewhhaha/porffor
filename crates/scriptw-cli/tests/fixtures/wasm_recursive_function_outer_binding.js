function helper() {
  return true;
}

function recurse(n) {
  if (n === 0) return helper();
  return recurse(n - 1);
}

if (recurse(1) !== true) throw "recursive function lost its outer lexical environment";
true;
