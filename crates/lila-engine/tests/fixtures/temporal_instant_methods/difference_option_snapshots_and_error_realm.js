const foreign = __lilaCreateRealm().global;
const foreignPrototype = foreign.Temporal.Instant.prototype;
const foreignRangeErrorPrototype = foreign.RangeError.prototype;
const receiver = new Temporal.Instant(0n);
const other = new Temporal.Instant(1500n);
foreign.RangeError = function () { throw 'public RangeError'; };
for (const difference of [foreignPrototype.until, foreignPrototype.since]) {
  let unit = 'week';
  let largestReads = 0;
  let smallestReads = 0;
  const options = {
    get largestUnit() { largestReads++; return unit; },
    get roundingIncrement() { unit = 'hour'; return 1; },
    get roundingMode() { return 'trunc'; },
    get smallestUnit() {
      smallestReads++;
      // A nested call cannot replace the borrowed method's error Realm.
      receiver.until(other, { largestUnit: 'second' });
      return 'nanosecond';
    }
  };
  let caught;
  try { difference.call(receiver, other, options); } catch (error) { caught = error; }
  if (!caught || Object.getPrototypeOf(caught) !== foreignRangeErrorPrototype) throw 'post-read RangeError Realm';
  if (unit !== 'hour' || largestReads !== 1 || smallestReads !== 1) throw 'retain largest-unit snapshot';
}
for (const difference of [Temporal.Instant.prototype.until, Temporal.Instant.prototype.since]) {
  const options = {
    largestUnit: 'microsecond',
    roundingIncrement: 1,
    roundingMode: 'trunc',
    get smallestUnit() {
      options.largestUnit = 'day';
      options.roundingIncrement = 0;
      options.roundingMode = 'ceil';
      return 'microsecond';
    }
  };
  const result = difference.call(receiver, other, options);
  const expected = difference === Temporal.Instant.prototype.until ? 1 : -1;
  if (result.microseconds !== expected || result.nanoseconds !== 0) throw 'later mutation must not replace converted settings';
}
print('ok');
