function make() {
  let o = { items: [{ x: 1 }, { x: 3 }] };
  return o;
}

let o = { inner: { x: 1 } };
o.inner.x = 4;

let a = [1, 2, 3];

make().items[1].x + o.inner.x + a.length;
