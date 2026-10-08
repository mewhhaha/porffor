function check(condition, label) {
  if (!condition) throw label;
}

var object = Object.create(null);
object.key = 1;
var let, value;
for (let in object) ;
check(let === "key", "identifier named let");

Object.defineProperty(Array.prototype, "1", {
  configurable: true,
  set: function (assigned) { value = assigned; }
});
for ([let][1] in object) ;
check(value === "key", "temporary array inherited setter");
for ([let][1] of ["next"]) ;
check(value === "next", "for-of temporary array inherited setter");
delete Array.prototype[1];

var array = [];
for (array.length in { "3": true }) ;
check(array.length === 3, "array length exotic setter");
for (array["named"] of ["value"]) ;
check(array.named === "value", "array named reference");

function mapped(parameter) {
  for (arguments[0] in object) ;
  check(parameter === "key", "mapped arguments indexed reference");
  for (arguments.extra of ["extra"]) ;
  check(arguments.extra === "extra", "arguments named reference");
  var key = Symbol("key");
  for (arguments[key] in object) ;
  check(arguments[key] === "key", "arguments symbol reference");
}
mapped(0);

var evaluations = 0;
var targets = [[], []];
function target() { return targets[evaluations++]; }
for (target()[0] of ["first", "second"]) ;
check(evaluations === 2 && targets[0][0] === "first" && targets[1][0] === "second",
  "reference base evaluated per iteration");

var sentinel = {};
var closed = 0;
Object.defineProperty(array, "blocked", { set: function () { throw sentinel; } });
var iterable = {
  [Symbol.iterator]: function () { return this; },
  next: function () { return { value: 1, done: false }; },
  return: function () { closed++; return {}; }
};
try {
  for (array.blocked of iterable) ;
  throw "setter must throw";
} catch (caught) {
  check(caught === sentinel && closed === 1, "abrupt setter closes iterator");
}

true;
