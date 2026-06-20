let seen = "";
let sawReceiverLength = false;

Array.prototype.forEach.call("ab", function(value, index, receiver) {
  seen = seen + value;
  if (index === 1) {
    sawReceiverLength = receiver.length === 2;
  }
});

let booleanCalled = false;
let trueResult = Array.prototype.forEach.call(true, function() {
  booleanCalled = true;
});
let falseResult = Array.prototype.forEach.call(false, function() {
  booleanCalled = true;
});

if (seen !== "ab") throw "forEach primitive string values";
if (!sawReceiverLength) throw "forEach primitive string receiver";
if (booleanCalled) throw "forEach boolean receiver callback";
if (trueResult !== undefined) throw "forEach true receiver result";
if (falseResult !== undefined) throw "forEach false receiver result";

true;
