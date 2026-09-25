function same(actual, expected) {
  if (actual !== expected) throw `${actual} !== ${expected}`;
}
function rangeError(action) {
  let caught = false;
  try { action(); } catch (error) {
    if (!(error instanceof RangeError)) throw error;
    caught = true;
  }
  if (!caught) throw 'expected RangeError';
}
const maximum = Number.MAX_SAFE_INTEGER;
for (const sign of [1, -1]) {
  const duration = new Temporal.Duration(0, 0, 0, 0, 0, 0, maximum * sign, 999 * sign);
  rangeError(() => duration.toString({ smallestUnit: 'seconds', roundingMode: 'expand' }));
  same(duration.toString({ smallestUnit: 'seconds', roundingMode: 'trunc' }),
    (sign < 0 ? '-' : '') + 'PT9007199254740991S');
  const withDay = new Temporal.Duration(0, 0, 0, sign, 0, 0,
    (maximum - 86400) * sign, 0, 0, 999999999 * sign);
  rangeError(() => withDay.toString({ fractionalSecondDigits: 7, roundingMode: 'expand' }));
  same(withDay.toString({ fractionalSecondDigits: 9 }),
    (sign < 0 ? '-' : '') + 'P1DT9007199254654591.999999999S');
}
const observed = [];
function option(name, value) {
  observed.push('get ' + name);
  return {
    get toString() {
      observed.push('get ' + name + '.toString');
      return function () {
        observed.push('call ' + name + '.toString');
        return value;
      };
    }
  };
}
const options = {
  get fractionalSecondDigits() { return option('fractionalSecondDigits', 'auto'); },
  get roundingMode() { return option('roundingMode', 'expand'); },
  get smallestUnit() { return option('smallestUnit', 'seconds'); }
};
rangeError(() => new Temporal.Duration(0, 0, 0, 0, 0, 0, maximum, 1).toString(options));
same(observed.join('|'), [
  'get fractionalSecondDigits', 'get fractionalSecondDigits.toString', 'call fractionalSecondDigits.toString',
  'get roundingMode', 'get roundingMode.toString', 'call roundingMode.toString',
  'get smallestUnit', 'get smallestUnit.toString', 'call smallestUnit.toString'
].join('|'));
print('ok');
