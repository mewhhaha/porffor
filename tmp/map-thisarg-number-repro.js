function callbackfn(val, idx, obj) {
  return this.valueOf() === 101;
}

var testResult = [11].map(callbackfn, 101);
testResult[0];
