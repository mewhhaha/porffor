function same(actual, expected, label) {
  if (actual !== expected) throw new Error(label);
}
var date = new Temporal.PlainDate(2000, 10, 29, 'buddhist');
var unread = {
  get disambiguation() { throw new Error('PlainDate has no options parameter'); },
  get offset() { throw new Error('second argument offset must be ignored'); },
  get overflow() { throw new Error('second argument overflow must be ignored'); }
};
var trace = '';
var result = date.toZonedDateTime({
  get timeZone() { trace += 'Z'; return 'America/Los_Angeles'; },
  get plainTime() {
    trace += 'T';
    return {
      get hour() { trace += 'H'; return 1; },
      get minute() { trace += 'M'; return 45; }
    };
  }
}, unread);
same(trace, 'ZTHM', 'zone conversion precedes plainTime conversion');
same(result.epochNanoseconds, 972809100000000000n, 'explicit time uses compatible fold');
same(result.calendarId, 'buddhist', 'actual PlainDate calendar survives');
same(result.toPlainDate().calendarId, 'buddhist', 'calendar survives record projection');
same(new Temporal.PlainDate(1970, 1, 1).toZonedDateTime('UTC', unread).epochNanoseconds,
  0n, 'string shorthand also ignores the second argument');

var reads = 0;
var unknownCaught = false;
try {
  date.toZonedDateTime({
    timeZone: 'Unknown/NotAZone',
    get plainTime() { reads++; return '01:45'; }
  }, unread);
} catch (error) { unknownCaught = error instanceof RangeError; }
same(unknownCaught, true, 'unknown name remains a catchable RangeError');
same(reads, 0, 'unknown zone is rejected before plainTime Get');

var marker = {};
var caught;
try {
  date.toZonedDateTime({
    timeZone: 'America/Los_Angeles', get plainTime() { throw marker; }
  });
} catch (error) { caught = error; }
same(caught, marker, 'plainTime abrupt completion preserves identity');
print('ok');
262;
