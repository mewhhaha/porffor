var ok = true;

var descriptorObject = { value: 1 };
var descriptor = Object.getOwnPropertyDescriptor(
  descriptorObject,
  "value",
  (descriptorObject.value = "s")
);
ok = ok && descriptor.value + 1 === "s1";

var prototype = { value: 1 };
var created = Object.create(
  prototype,
  undefined,
  (prototype.value = "s")
);
ok = ok && created.value + 1 === "s1";

var receiver = {
  value: 1,
  read: function() { return this.value; }
};
ok = ok && receiver.read(receiver.value = "s") + 1 === "s1";

var defaultValue = 1;
function readDefaultValue() {
  return this.defaultValue;
}
ok = ok && readDefaultValue(defaultValue = "s") + 1 === "s1";

class PrivateReceiver {
  #read() { return this.value; }

  readAfterArgument() {
    return this.#read(this.value = "s") + 1;
  }
}
var privateReceiver = new PrivateReceiver();
privateReceiver.value = 1;
ok = ok && privateReceiver.readAfterArgument() === "s1";

var optionalReceiver = {
  value: 1,
  read: function() { return this.value; }
};
ok = ok && optionalReceiver?.read(optionalReceiver.value = "s") + 1 === "s1";

function Constructor() {}
Constructor.prototype = { value: 1 };
var instance = new Constructor(Constructor.prototype = { value: "s" });
ok = ok && instance.value + 1 === "s1";

var getterState = { value: 1 };
var getterReceiver = {
  get read() {
    getterState.value = "s";
    return function(value) { return value; };
  }
};
ok = ok && getterReceiver?.read(
  Object.getOwnPropertyDescriptor(getterState, "value").value
) + 1 === "s1";

ok;
