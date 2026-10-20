let desc = Object.getOwnPropertyDescriptor(DataView.prototype, "byteLength");
let lengthDesc = Object.getOwnPropertyDescriptor(desc.get, "length");

Object.getOwnPropertyDescriptor(desc.get, "name");

lengthDesc.value;
