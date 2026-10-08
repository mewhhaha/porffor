function check(value, message) { if (!value) throw new Error(message); }
function retained() {
  var string = 'a\u0301\ud83d\ude00\ud800';
  var segments = new Intl.Segmenter('en').segment(string);
  return { segments: segments, iterator: segments[Symbol.iterator]() };
}
var owner = retained();
var finite = owner.segments.containing(2), absent = owner.segments.containing(Infinity);
var saved = [finite, absent];
check(typeof saved[0] === 'object' && typeof saved[1] === 'undefined', 'stored result tags');
check(String(saved[1]) === 'undefined' && saved[1] === undefined, 'undefined coercion');
check(saved[0].segment.length === 2 && saved[0].index === 2, 'object survives storage');
var pressure = [];
for (var i = 0; i < 1000; ++i) pressure.push({ text: 'allocation-' + i, values: [i, i + 1] });
check(owner.segments.containing(4).segment.charCodeAt(0) === 0xd800, 'retained source after allocations');
owner.segments = null;
check(owner.iterator.next().value.segment === 'a\u0301', 'iterator retains original combining sequence');
check(owner.iterator.next().value.segment.length === 2, 'iterator retains supplementary pair');
check(owner.iterator.next().value.segment.charCodeAt(0) === 0xd800, 'iterator retains isolated surrogate');
check(owner.iterator.next().done, 'retained iterator completes');
print('ok containing_tagged_storage_and_retained_owners'); 262;
