var arr = new Array(10);
function f() {
  arr.map(foo);
}
try {
  f();
  "no throw";
} catch (e) {
  e.name;
}
