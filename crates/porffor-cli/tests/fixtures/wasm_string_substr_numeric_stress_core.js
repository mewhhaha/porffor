var ok = true;

function check(value) {
  ok = ok && value;
}

var starts = [-Infinity, -9.5, -3, -0.5, -0, 0, 1.5, 3, Infinity, NaN];
var lengths = [-Infinity, -2, -0.5, 0, 1.5, 4, Infinity, NaN, undefined];
var moreStarts = [10].map(function(value) { return value - 8; });
var allStarts = [-12, ...starts, ...moreStarts];
var strings = ["abcdef", "0123456789"];

check(Number.isNaN(NaN));
check(!Number.isNaN("NaN"));
check(Math.trunc(1.9) === 1);
check(Math.trunc(-1.9) === -1);
check(Math.min(5, -2) === -2);
check(Math.max(5, -2) === 5);

function refSubstr(str, start, length) {
  var size = str.length;
  var intStart = Number.isNaN(start) ? 0 : Math.trunc(start);
  if (intStart === Infinity) intStart = size;
  if (intStart === -Infinity) intStart = 0;
  if (intStart < 0) intStart = Math.max(size + intStart, 0);
  if (intStart > size) intStart = size;
  var intLength = length === undefined ? size - intStart : (Number.isNaN(length) ? 0 : Math.trunc(length));
  if (intLength === Infinity) intLength = size - intStart;
  if (intLength === -Infinity || intLength < 0) intLength = 0;
  var end = Math.min(size, intStart + intLength);
  return str.substring(intStart, end);
}

for (let str of strings) {
  for (let start of allStarts) {
    for (let length of lengths) {
      check(str.substr(start, length) === refSubstr(str, start, length));
    }
  }
}

check("abcdef".substring(1, 4) === "bcd");
check("abcdef"[2] === "c");

ok;
