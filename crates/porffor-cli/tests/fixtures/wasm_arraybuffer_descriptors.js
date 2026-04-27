let desc = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, "byteLength");
let lengthDesc = Object.getOwnPropertyDescriptor(desc.get, "length");

lengthDesc.value;
