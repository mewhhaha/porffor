function check(actual, expected, label) {
  if (actual !== expected) throw new Error(label + ': ' + actual + ' != ' + expected);
}
function orderedBag(calendar, trace) {
  var values = {
    calendar: calendar, day: 14, era: 'ce', eraYear: 2021, hour: 12,
    microsecond: 0, millisecond: 0, minute: 0, month: 3, monthCode: 'M03',
    nanosecond: 0, offset: '-04:00', second: 0,
    timeZone: 'America/New_York', year: 2021
  };
  var bag = {};
  Object.keys(values).forEach(function(key) {
    Object.defineProperty(bag, key, {get: function() {
      trace.push('get ' + key);
      var value = values[key];
      if (key === 'calendar' || key === 'timeZone') return value;
      if (key === 'offset') return {[Symbol.toPrimitive]: function(hint) {
        trace.push('offset ' + hint);
        return value;
      }};
      if (typeof value === 'string') return {toString: function() {
        trace.push('string ' + key);
        return value;
      }};
      return {valueOf: function() {trace.push('number ' + key); return value;}};
    }});
  });
  return bag;
}
var common = 'get hour|number hour|get microsecond|number microsecond|get millisecond|number millisecond|get minute|number minute|get month|number month|get monthCode|string monthCode|get nanosecond|number nanosecond|get offset|offset string|get second|number second|get timeZone|get year|number year';
var isoTrace = [];
var iso = orderedBag('iso8601', isoTrace);
check(Temporal.Duration.from({days: 1}).total({unit: 'hours', relativeTo: iso}), 24, 'ISO elapsed day');
check(isoTrace.join('|'), 'get calendar|get day|number day|' + common, 'ISO read order and ignored eras');
var eraTrace = [];
var gregory = orderedBag('gregory', eraTrace);
check(Temporal.Duration.from({days: 1}).total({unit: 'hours', relativeTo: gregory}), 24, 'Gregorian elapsed day');
check(eraTrace.join('|'), 'get calendar|get day|number day|get era|string era|get eraYear|number eraYear|' + common, 'era read order');
262;
