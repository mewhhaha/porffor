let seenValue = false;
let seenIndex = false;
let seenReceiver = false;

function callback(value, index, receiver) {
  if (index === 0) {
    seenValue = value === 11;
    seenIndex = true;
    seenReceiver = receiver.length === 20;
  }
}

let obj = {
  10: 10,
  length: 20
};

Object.defineProperty(obj, "0", {
  get: function() {
    return 11;
  },
  configurable: true
});

Array.prototype.forEach.call(obj, callback);

if (!seenIndex) throw "forEach callback index";
if (!seenValue) throw "forEach callback value";
if (!seenReceiver) throw "forEach callback receiver";

true;
