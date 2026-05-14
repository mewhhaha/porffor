function check(TA) {
  let result = TA.from([1]);
  if (result.length !== 1) throw "dynamic TypedArray.from length";
  if (result[0] !== 1) throw "dynamic TypedArray.from value";
  if (result.constructor !== TA) throw "dynamic TypedArray.from constructor";
  if (Object.getPrototypeOf(result) !== TA.prototype) {
    throw "dynamic TypedArray.from prototype";
  }
}

check(Uint8Array);
check(Float32Array);

262;
