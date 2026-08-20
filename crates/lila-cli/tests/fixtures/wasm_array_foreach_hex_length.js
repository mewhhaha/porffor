var seen = "";

function callback(value, index) {
  seen = seen + String(index) + ":" + String(value) + ";";
}

var obj = {
  1: 11,
  2: 9,
  length: "0x0002"
};

Array.prototype.forEach.call(obj, callback);

seen === "1:11;";
