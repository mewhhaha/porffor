function expectEpochNanoseconds(milliseconds, expected) {
  const instant = new Date(milliseconds).toTemporalInstant();
  if (instant.epochNanoseconds !== expected) throw milliseconds;
}

function expectError(errorConstructor, callback) {
  let thrown = false;
  try {
    callback();
  } catch (error) {
    if (!(error instanceof errorConstructor)) throw error;
    thrown = true;
  }
  if (!thrown) throw errorConstructor.name;
}

const toTemporalInstant = Date.prototype.toTemporalInstant;
if (typeof toTemporalInstant !== "function") throw "missing";
if (toTemporalInstant.name !== "toTemporalInstant") throw "name";
if (toTemporalInstant.length !== 0) throw "length";

expectEpochNanoseconds(0, 0n);
expectEpochNanoseconds(123456789, 123456789000000n);
expectEpochNanoseconds(-123456789, -123456789000000n);
expectEpochNanoseconds(-8640000000000000, -8640000000000000000000n);
expectEpochNanoseconds(8640000000000000, 8640000000000000000000n);

expectError(RangeError, function() {
  new Date(NaN).toTemporalInstant();
});
expectError(TypeError, function() {
  toTemporalInstant.call({});
});
expectError(TypeError, function() {
  toTemporalInstant.call(undefined);
});
expectError(TypeError, function() {
  new toTemporalInstant();
});

262;
