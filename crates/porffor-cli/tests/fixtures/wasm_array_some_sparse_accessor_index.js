let kValue = "abc";
let arr = [];

Object.defineProperty(arr, "10", {
  get: function () {
    return kValue;
  },
  configurable: true
});

if (arr.length !== 11) throw "length";
if (arr[10] !== kValue) throw "direct get";

let seen = "unset";
let calls = 0;
let result = arr.some(function (value, index) {
  calls++;
  if (index === 10) {
    seen = value;
    return value === kValue;
  }
  return false;
});

if (seen !== kValue) throw "seen";
if (calls < 1) throw "calls";

result;
