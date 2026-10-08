function same(actual, expected, label) {
  if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual + ' != ' + expected);
}
function range(action) {
  try {action();} catch (caught) {if (!(caught instanceof RangeError)) throw caught; return;}
  throw new Error('missing RangeError');
}
// Source names only Duration. These calendar paths must work in the initial
// compiler pass before namespace bootstrap materializes carrier constructors.
for (const calendar of ['chinese', 'dangi']) {
  const oneYear = Temporal.Duration.from({years: 1});
  const twoYears = Temporal.Duration.from({years: 2});
  const twelve = Temporal.Duration.from({months: 12});
  const thirteen = Temporal.Duration.from({months: 13});
  const twentyFive = Temporal.Duration.from({months: 25});
  const common = {calendar, year: 2000, monthCode: 'M04', day: 1};
  const leap = {calendar, year: 2001, monthCode: 'M04', day: 1};
  const leapYear = {calendar, year: 2020, monthCode: 'M01', day: 1};
  same(oneYear.total({unit: 'months', relativeTo: common}), 12, 'common-year actual month span');
  same(oneYear.total({unit: 'months', relativeTo: leap}), 13, 'leap-year actual month span');
  same(twoYears.total({unit: 'months', relativeTo: common}), 25, 'two-year serial span');
  same(Temporal.Duration.compare(twelve, oneYear, {relativeTo: common}), 0, 'common year versus twelve');
  same(Temporal.Duration.compare(twelve, oneYear, {relativeTo: leap}), -1, 'leap year exceeds twelve');
  same(Temporal.Duration.compare(thirteen, oneYear, {relativeTo: leap}), 0, 'leap year versus thirteen');
  for (const relativeTo of [
    '2000-01-01[u-ca=' + calendar + ']',
    '2000-01-01T12:00[+05:30][u-ca=' + calendar + ']',
    '2000-01-01T12:00[America/New_York][u-ca=' + calendar + ']'
  ]) {
    same(twentyFive.total({unit: 'years', relativeTo}), 2, 'plain/zoned string actual two years');
    same(Temporal.Duration.compare(twentyFive, twoYears, {relativeTo}), 0, 'plain/zoned string actual pair equality');
    // Duration add/subtract reject calendar units and have no relativeTo
    // option. An extra argument must not be consulted to permit those units.
    let relativeReads = 0;
    const unusedOptions = {get relativeTo() {relativeReads++; return relativeTo;}};
    range(() => thirteen.add(twelve, unusedOptions));
    range(() => twentyFive.subtract(twelve, unusedOptions));
    same(relativeReads, 0, 'Duration add/subtract ignore extra options');
  }
  for (const relativeTo of [
    leapYear,
    {...leapYear, hour: 12, timeZone: 'UTC'},
    {...leapYear, hour: 12, timeZone: '+05:30'},
    {...leapYear, hour: 12, timeZone: 'America/New_York'}
  ]) {
    same(oneYear.total({unit: 'days', relativeTo}), 384, 'pinned actual leap-year days');
    same(thirteen.total({unit: 'days', relativeTo}), 384, 'actual months and year agree');
    same(thirteen.total({unit: 'years', relativeTo}), 1, 'actual year unit');
    same(Temporal.Duration.from({days: 384}).round({largestUnit: 'years', smallestUnit: 'years', relativeTo}).years, 1, 'actual year rounding anchor');
    same(Temporal.Duration.from({days: 383}).round({smallestUnit: 'years', roundingMode: 'floor', relativeTo}).years, 0, 'actual year lower bracket');
    same(Temporal.Duration.from({days: 383}).round({smallestUnit: 'years', roundingMode: 'ceil', relativeTo}).years, 1, 'actual year upper bracket');
  }
  same(Temporal.Duration.from({months: -13}).total({unit: 'years', relativeTo: {calendar, year: 2002, monthCode: 'M04', day: 1}}), -1, 'negative actual year span');
  let log = [];
  same(oneYear.total({get relativeTo() {log.push('relative'); return leapYear;}, get unit() {log.push('unit'); return 'days';}}), 384, 'observed relative anchor');
  same(log.join('|'), 'relative|unit', 'relative option order');
  const marker = {};
  try {
    oneYear.round({get relativeTo() {throw marker;}, get roundingIncrement() {log.push('late'); return 1;}});
    throw new Error('missing original relative throw');
  } catch (caught) {if (caught !== marker) throw new Error('original relative throw identity');}
  same(log.join('|'), 'relative|unit', 'no later options after abrupt relative');
}
print('east-asian-relative-duration:ok');
262;
