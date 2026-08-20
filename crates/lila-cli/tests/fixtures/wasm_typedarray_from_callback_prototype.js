let constructors = [Uint8Array, Float32Array];

function testWithTypedArrayConstructors(callback) {
  for (let index = 0; index < constructors.length; index = index + 1) {
    callback(constructors[index]);
  }
}

testWithTypedArrayConstructors(function (TA) {
  let result = TA.from([]);
  if (result.constructor !== TA) throw "callback constructor identity";
  if (Object.getPrototypeOf(result) !== TA.prototype) {
    throw "callback prototype identity";
  }
});

262;
