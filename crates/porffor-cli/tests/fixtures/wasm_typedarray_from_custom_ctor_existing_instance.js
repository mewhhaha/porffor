var iteratorInstance = new Uint8Array(3);
var iteratorCalled = 0;

function iteratorCtor(len) {
  iteratorCalled++;
  if (len !== 3) throw "iterator existing length";
  return iteratorInstance;
}

var iteratorResult = Uint8Array.from.call(iteratorCtor, [9, 8, 7]);

if (iteratorCalled !== 1) throw "iterator existing called";
if (iteratorResult !== iteratorInstance) throw "iterator existing identity";
if (iteratorResult[0] !== 9) throw "iterator existing first";
if (iteratorResult[1] !== 8) throw "iterator existing second";
if (iteratorResult[2] !== 7) throw "iterator existing third";

var arrayLikeInstance = new Uint8Array(2);
var arrayLikeCalled = 0;

function arrayLikeCtor(len) {
  arrayLikeCalled++;
  if (len !== 2) throw "array-like existing length";
  return arrayLikeInstance;
}

var arrayLikeResult = Uint8Array.from.call(arrayLikeCtor, { 0: 4, 1: 3, length: 2 });

if (arrayLikeCalled !== 1) throw "array-like existing called";
if (arrayLikeResult !== arrayLikeInstance) throw "array-like existing identity";
if (arrayLikeResult[0] !== 4) throw "array-like existing first";
if (arrayLikeResult[1] !== 3) throw "array-like existing second";

262;
