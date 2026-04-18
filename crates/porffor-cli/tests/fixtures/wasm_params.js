function outer(x = 1, ...xs) {
  return (() => arguments[0])();
}

let o = {
  x: 2,
  f(y = this.x) {
    return y;
  }
};

outer(3, 4, 5);
o.f();
