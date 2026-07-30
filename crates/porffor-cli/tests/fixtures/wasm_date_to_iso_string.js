function expect(date, expected) {
  if (date.toISOString() !== expected) throw expected;
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

expect(new Date(0), "1970-01-01T00:00:00.000Z");
expect(new Date(Date.UTC(1999, 9, 10, 10, 10, 10, 10)), "1999-10-10T10:10:10.010Z");

let date = new Date(0);
date.setUTCFullYear(20);
expect(date, "0020-01-01T00:00:00.000Z");

date = new Date(0);
date.setUTCFullYear(-1);
expect(date, "-000001-01-01T00:00:00.000Z");

date = new Date(0);
date.setUTCFullYear(12345);
expect(date, "+012345-01-01T00:00:00.000Z");

date = new Date(0);
date.setUTCFullYear(-12345);
expect(date, "-012345-01-01T00:00:00.000Z");

expect(new Date(8640000000000000), "+275760-09-13T00:00:00.000Z");
expect(new Date(-8640000000000000), "-271821-04-20T00:00:00.000Z");
expectError(RangeError, function() {
  new Date(NaN).toISOString();
});
expectError(TypeError, function() {
  Date.prototype.toISOString.call({});
});

262;
