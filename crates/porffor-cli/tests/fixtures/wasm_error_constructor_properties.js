var message = "my-message";
var cause = { message: "my-cause" };
var error = new Error(message, { cause: cause });
var messageDesc = Object.getOwnPropertyDescriptor(error, "message");
var causeDesc = Object.getOwnPropertyDescriptor(error, "cause");
var globalDesc = Object.getOwnPropertyDescriptor(this, "Error");
var prototypeDesc = Object.getOwnPropertyDescriptor(Error, "prototype");

var ok =
  messageDesc.value === message &&
  messageDesc.writable === true &&
  messageDesc.enumerable === false &&
  messageDesc.configurable === true &&
  causeDesc.value === cause &&
  causeDesc.writable === true &&
  causeDesc.enumerable === false &&
  causeDesc.configurable === true &&
  Object.getOwnPropertyDescriptor(new Error(message), "cause") === undefined &&
  Object.getOwnPropertyDescriptor(new Error(message, { cause: undefined }), "cause").value === undefined &&
  globalDesc.value === Error &&
  globalDesc.writable === true &&
  globalDesc.enumerable === false &&
  globalDesc.configurable === true &&
  Error.prototype.isPrototypeOf(new Error()) === true &&
  Error.prototype.isPrototypeOf(Error()) === true &&
  prototypeDesc.value === Error.prototype &&
  prototypeDesc.writable === false &&
  prototypeDesc.enumerable === false &&
  prototypeDesc.configurable === false;

ok;
