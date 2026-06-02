var arr = new Array(10);
try {
  arr.map(foo);
  "no throw";
} catch (e) {
  e instanceof ReferenceError;
}
