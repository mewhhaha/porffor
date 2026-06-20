foo.prototype = new Array(1, 2, 3);

function foo() {}

var f = new foo();
f.length = 1;

var callCount = 0;
var seen = "";

function callback(value, index) {
  callCount++;
  seen = seen + String(index) + ":" + String(value) + ";";
}

var result = f.forEach(callback);

callCount === 1 && seen === "0:1;" && result === undefined;
