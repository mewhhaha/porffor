let constructors = [Uint8Array, Float32Array];
let arraySource = [42, 43, 42];
let objectSource = { length: 3, 0: 1, 2: 3 };

function testWithTypedArrayConstructors(callback) {
  for (let index = 0; index < constructors.length; index = index + 1) {
    callback(constructors[index]);
  }
}

testWithTypedArrayConstructors(function (TA) {
  let result = TA.from(arraySource);
  if (result.length !== 3) throw "callback captured array length";
  if (result[0] !== 42 || result[1] !== 43 || result[2] !== 42) {
    throw "callback captured array values";
  }
});

testWithTypedArrayConstructors(function (TA) {
  let result = TA.from(objectSource);
  if (result.length !== 3) throw "callback captured object length";
  if (result[0] !== 1 || result[2] !== 3) throw "callback captured object values";
  if (TA === Float32Array) {
    if (result[1] === result[1]) throw "callback captured object float missing";
  } else {
    if (result[1] !== 0) throw "callback captured object int missing";
  }
});

262;
