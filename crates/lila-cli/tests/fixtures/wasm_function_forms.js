function make() {
  return () => this.x;
}

let add = (x, y) => x + y;
let fact = function fact(n) {
  if (n === 0) {
    return 1;
  }

  return n * fact(n - 1);
};
let o = { x: 3, f: make };
let g = o.f();

add(2, 3) + fact(4) + g();
