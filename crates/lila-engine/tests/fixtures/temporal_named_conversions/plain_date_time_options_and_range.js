function same(actual, expected, label) {
  if (actual !== expected) throw new Error(label);
}
var trace = '';
var options = {
  get disambiguation() {
    trace += 'D';
    return {
      get toString() {
        trace += 'T';
        return function () { trace += 'S'; return 'later'; };
      }
    };
  },
  get offset() { throw new Error('offset is not a PlainDateTime conversion option'); },
  get overflow() { throw new Error('overflow is not a PlainDateTime conversion option'); }
};
var fold = new Temporal.PlainDateTime(2000, 10, 29, 1, 45);
same(fold.toZonedDateTime('America/Los_Angeles', options).epochNanoseconds,
  972812700000000000n, 'retained converted option selects later');
same(trace, 'DTS', 'only disambiguation is read and converted once');

trace = '';
var boundaryCaught = false;
try {
  new Temporal.PlainDateTime(-271821, 4, 20).toZonedDateTime('+23:59', options);
} catch (error) { boundaryCaught = error instanceof RangeError; }
same(boundaryCaught, true, 'selected epoch outside Instant range');
same(trace, 'DTS', 'options complete before algorithmic epoch validation');

trace = '';
var unknownCaught = false;
try { fold.toZonedDateTime('Unknown/NotAZone', options); }
catch (error) { unknownCaught = error instanceof RangeError; }
same(unknownCaught, true, 'unknown zone error');
same(trace, '', 'zone conversion precedes options');

var marker = {};
var caught;
try {
  fold.toZonedDateTime('America/Los_Angeles', {get disambiguation() { throw marker; }});
} catch (error) { caught = error; }
same(caught, marker, 'disambiguation getter abrupt identity');

var foreign = __lilaCreateRealm().global;
var foreignMethod = foreign.Temporal.PlainDateTime.prototype.toZonedDateTime;
var foreignRangeErrorPrototype = foreign.RangeError.prototype;
foreign.RangeError = function () { throw new Error('public constructor is not the intrinsic'); };
caught = undefined;
try {
  foreignMethod.call(fold, 'America/Los_Angeles', {
    get disambiguation() {
      fold.toZonedDateTime('UTC');
      return 'reject';
    }
  });
} catch (error) { caught = error; }
same(Object.getPrototypeOf(caught), foreignRangeErrorPrototype,
  'inverse rejection uses the builtin function realm after a nested getter call');
print('ok');
262;
