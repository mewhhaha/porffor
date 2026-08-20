let arrayBuffer = new ArrayBuffer(8);
let log = "";
let start = {
  valueOf: function() {
    log += "start-";
    return 0;
  }
};
let end = {
  valueOf: function() {
    log += "end";
    return 8;
  }
};

let sliced = arrayBuffer.slice(start, end);
if (log !== "start-end") throw "slice conversion order";
if (sliced.byteLength !== 8) throw "slice conversion length";

123;
