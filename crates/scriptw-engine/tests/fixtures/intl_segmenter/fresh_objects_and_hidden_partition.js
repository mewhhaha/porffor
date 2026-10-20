function check(value, message) { if (!value) throw new Error(message); }
var segmenter = new Intl.Segmenter('en', { granularity: 'word' });
var segments = segmenter.segment('Hi there');
var a = segments.containing(0), b = segments.containing(0);
check(a !== b && a.segment === b.segment && Object.getPrototypeOf(a) === Object.prototype, 'fresh containing object');
for (var key of Object.keys(a)) {
  var d = Object.getOwnPropertyDescriptor(a, key);
  check(d.writable && d.enumerable && d.configurable, 'ordinary data flags');
}
a.segment = 'corrupted'; a.index = 99; delete a.input; a.isWordLike = false;
segmenter.granularity = 'sentence'; segmenter.locale = 'bad';
segments.boundaries = [99]; segments.input = 'bad';
check(segments.containing(0).segment === 'Hi' && segments.containing(0).input === 'Hi there', 'inaccessible partition/source');
var first = segments[Symbol.iterator](), second = segments[Symbol.iterator]();
var x = first.next(), y = second.next();
check(first !== second && x !== y && x.value !== y.value && x.value.segment === 'Hi', 'fresh independent iterators and results');
x.value.segment = 'bad'; check(second.next().value.segment === ' ', 'result mutation does not alter next partition');
check(Object.keys(segments).join(',') === 'boundaries,input', 'no own hidden slot properties');
print('ok fresh_objects_and_hidden_partition'); 262;
