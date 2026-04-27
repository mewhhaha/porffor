let getSigned = DataView.prototype.getBigInt64;
let setSigned = DataView.prototype.setBigInt64;
let getUnsigned = DataView.prototype.getBigUint64;
let setUnsigned = DataView.prototype.setBigUint64;

__porfAssertThrows(TypeError, function () {
  new getSigned();
});

__porfAssertThrows(TypeError, function () {
  new setUnsigned();
});

(getSigned.length === 1) +
  (getSigned.name === "getBigInt64") +
  (setSigned.length === 2) +
  (setSigned.name === "setBigInt64") +
  (getUnsigned.length === 1) +
  (getUnsigned.name === "getBigUint64") +
  (setUnsigned.length === 2) +
  (setUnsigned.name === "setBigUint64");
