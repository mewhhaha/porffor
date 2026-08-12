let empty = new Array(0);
let one = new Array(1);
let dense = new Array(1000001);
let huge = new Array(4294967295);

empty[0] = 1;
one[0] = 2;
dense[1000000] = 3;
if (empty.length !== 1 || empty[0] !== 1 || one.length !== 1 || one[0] !== 2) {
  throw new Error("small Array lengths changed");
}
if (dense.length !== 1000001 || dense[1000000] !== 3) {
  throw new Error("dense-range Array write changed");
}
if (huge.length !== 4294967295 || huge[4294967294] !== undefined) {
  throw new Error("sparse Array length/read changed");
}
huge[7] = 42;
if (huge[7] !== 42 || huge.length !== 4294967295) {
  throw new Error("sparse Array write changed");
}

true;
