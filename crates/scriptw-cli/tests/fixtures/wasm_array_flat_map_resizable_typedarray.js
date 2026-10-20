var failures = 0;

var oddBuffer = new ArrayBuffer(4, { maxByteLength: 8 });
var oddView = new Uint16Array(oddBuffer);
oddView[0] = 11;
oddView[1] = 22;
oddBuffer.resize(5);
var oddCalls = 0;
var oddResult = Array.prototype.flatMap.call(oddView, function (value) {
  oddCalls += 1;
  return [value];
});
if (oddCalls !== 2 || oddResult.length !== 2 || oddResult[0] !== 11 || oddResult[1] !== 22) {
  failures |= 1;
}

var growBuffer = new ArrayBuffer(2, { maxByteLength: 6 });
var growView = new Uint16Array(growBuffer);
growView[0] = 31;
var growCalls = 0;
var growResult = Array.prototype.flatMap.call(growView, function (value, index) {
  growCalls += 1;
  if (index === 0) {
    growBuffer.resize(6);
    growView[1] = 32;
    growView[2] = 33;
  }
  return [value];
});
if (growCalls !== 1 || growResult.length !== 1 || growResult[0] !== 31) {
  failures |= 2;
}

var shrinkBuffer = new ArrayBuffer(6, { maxByteLength: 6 });
var shrinkView = new Uint16Array(shrinkBuffer);
shrinkView[0] = 41;
shrinkView[1] = 42;
shrinkView[2] = 43;
var shrinkCalls = 0;
var shrinkResult = Array.prototype.flatMap.call(shrinkView, function (value, index) {
  shrinkCalls += 1;
  if (index === 0) shrinkBuffer.resize(3);
  return [value];
});
if (shrinkCalls !== 1 || shrinkResult.length !== 1 || shrinkResult[0] !== 41) {
  failures |= 4;
}

var fixedBuffer = new ArrayBuffer(4, { maxByteLength: 8 });
var fixedView = new Uint16Array(fixedBuffer, 2, 1);
fixedView[0] = 51;
fixedBuffer.resize(1);
var fixedOutOfBoundsCalls = 0;
var fixedOutOfBoundsResult = Array.prototype.flatMap.call(fixedView, function (value) {
  fixedOutOfBoundsCalls += 1;
  return [value];
});
if (fixedOutOfBoundsCalls !== 0 || fixedOutOfBoundsResult.length !== 0) {
  failures |= 8;
}

fixedBuffer.resize(4);
fixedView[0] = 52;
var fixedRegrownCalls = 0;
var fixedRegrownResult = Array.prototype.flatMap.call(fixedView, function (value) {
  fixedRegrownCalls += 1;
  return [value];
});
if (fixedRegrownCalls !== 1 || fixedRegrownResult.length !== 1 || fixedRegrownResult[0] !== 52) {
  failures |= 16;
}

var detachedBuffer = new ArrayBuffer(4);
var detachedView = new Uint16Array(detachedBuffer);
detachedView[0] = 61;
detachedBuffer.transfer();
var detachedCalls = 0;
var detachedResult = Array.prototype.flatMap.call(detachedView, function (value) {
  detachedCalls += 1;
  return [value];
});
if (detachedCalls !== 0 || detachedResult.length !== 0) {
  failures |= 32;
}

failures === 0;
