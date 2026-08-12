let transfer = ArrayBuffer.prototype.transfer;
let transferDesc = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, "transfer");
if (transferDesc === undefined) throw "transfer descriptor";


let transferToFixedLength = ArrayBuffer.prototype.transferToFixedLength;
let fixedDesc = Object.getOwnPropertyDescriptor(
  ArrayBuffer.prototype,
  "transferToFixedLength"
);
if (fixedDesc === undefined) throw "fixed descriptor";


let buffer = new ArrayBuffer(4);
let transferThrew = false;
try {
  new buffer.transfer();
} catch (error) {
  transferThrew = true;
}
if (!transferThrew) throw "transfer constructor";

let fixedThrew = false;
try {
  new buffer.transferToFixedLength();
} catch (error) {
  fixedThrew = true;
}
if (!fixedThrew) throw "fixed constructor";

123;
