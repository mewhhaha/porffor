function check(condition, message) {
  if (!condition) throw message;
}

var array = [1, 2, 3, 4, 5];
var joins = 0;
var origJoin = Array.prototype.join;
Array.prototype.join = function() {
  joins++;
  return origJoin.call(this);
};
var ta = new Float64Array(array);
check(joins === 0, "constructor must not join the source array, joins=" + joins);
check(ta.length === 5 && ta[4] === 5, "elements copied");

var strings = 0;
var origToString = Array.prototype.toString;
Array.prototype.toString = function() {
  strings++;
  return origToString.call(this);
};
var tb = new Int32Array(array);
check(strings === 0, "constructor must not stringify the source array");
check(tb.length === 5 && tb[0] === 1, "elements copied");

Array.prototype.join = origJoin;
Array.prototype.toString = origToString;
7;
