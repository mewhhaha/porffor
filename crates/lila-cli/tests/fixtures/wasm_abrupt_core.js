var x = 0;
try {
  x = 1;
} finally {
  x = x + 1;
}

var y = 0;
try {
  throw 1;
} catch (e) {
  y = 1;
} finally {
  y = y + 1;
}

let o = { x: 1 };
delete o.x;

let a = [1, 2];
delete a[0];

function F() {
  this.kind = typeof new.target;
  this.arrowKind = (() => typeof new.target)();
}

let f = new F();

(x === 2)
  && (y === 2)
  && !("x" in o)
  && !(0 in a)
  && (a.length === 2)
  && (f.kind === "function")
  && (f.arrowKind === "function");
