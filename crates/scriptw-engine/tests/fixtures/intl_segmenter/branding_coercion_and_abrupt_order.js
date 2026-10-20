function check(value, message) { if (!value) throw new Error(message); }
function throws(type, fn) { try { fn(); } catch (e) { check(e instanceof type, 'wrong exception'); return; } throw new Error('missing exception'); }
var segmenter = new Intl.Segmenter('en'), segments = segmenter.segment('AB');
var sp = Object.getPrototypeOf(segments), iterator = segments[Symbol.iterator](), ip = Object.getPrototypeOf(iterator);
var count = 0, input = { toString: function () { count++; return 'A\ud800'; } };
throws(TypeError, function () { Intl.Segmenter.prototype.segment.call({}, input); }); check(count === 0, 'brand before ToString');
var actual = segmenter.segment(input); check(count === 1 && actual.containing(1).segment.charCodeAt(0) === 0xd800, 'one lossless ToString');
var index = { valueOf: function () { count++; return 1; } };
throws(TypeError, function () { sp.containing.call({}, index); }); check(count === 1, 'Segments brand before ToInteger');
check(actual.containing(index).index === 1 && count === 2, 'one index conversion');
throws(TypeError, function () { sp[Symbol.iterator].call({}); });
throws(TypeError, function () { ip.next.call({}); });
throws(TypeError, function () { Intl.Segmenter.prototype.resolvedOptions.call(new Proxy(segmenter, {})); });
throws(TypeError, function () { ip.next.call(new Proxy(iterator, {})); });
var sentinel = {};
var caughtInput = false, caughtIndex = false;
try { segmenter.segment({ toString: function () { throw sentinel; } }); } catch (e) { check(e === sentinel, 'exact abrupt input'); caughtInput = true; }
try { actual.containing({ valueOf: function () { throw sentinel; } }); } catch (e) { check(e === sentinel, 'exact abrupt index'); caughtIndex = true; }
check(caughtInput && caughtIndex, 'both abrupt completions propagated');
check(iterator.next().value.segment === 'A', 'failed foreign next does not advance owner');
print('ok branding_coercion_and_abrupt_order'); 262;
