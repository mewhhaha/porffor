(function () {
  function check(actual, expected, name) { if (actual !== expected) throw new Error(name); }
  var text = 'ab\ud800cd\udc00';
  check(text.slice(0), text, 'whole slice');
  check(text.substring(0, text.length), text, 'whole substring');
  check(text.slice(2, 3), '\ud800', 'partial lone high surrogate');
  check(text.slice(5), '\udc00', 'partial lone low surrogate');
  check(text.slice(3, 3), '', 'empty partial slice');
  check(text.slice(text.length, text.length), '', 'empty slice at backing boundary');
  check(text.slice(100, 200), '', 'clamped empty slice');
  check(text.substring(3, 1), 'b\ud800', 'swapped substring');
  check(text.substr(2, 1), '\ud800', 'legacy substr units');
  check(text.charAt(100), '', 'out of bounds charAt');
  check(''.slice(0), '', 'empty full slice');
  var log = '';
  var receiver = {toString: function () { log += 'r'; return text; }};
  var start = {valueOf: function () { log += 's'; return 2; }};
  var end = {valueOf: function () { log += 'e'; return 3; }};
  check(String.prototype.slice.call(receiver, start, end), '\ud800', 'coerced slice');
  check(log, 'rse', 'slice coercion order');
  var spaces = ['\u0009','\u000b','\u000c','\u0020','\u00a0','\u1680','\u2000','\u2001','\u2002','\u2003','\u2004','\u2005','\u2006','\u2007','\u2008','\u2009','\u200a','\u202f','\u205f','\u3000','\ufeff','\u000a','\u000d','\u2028','\u2029'];
  for (var i=0;i<spaces.length;i++) {
    var s=spaces[i];
    check((s+text+s).trim(), text, 'both Unicode whitespace');
    check((s+s).trimStart(), '', 'all whitespace trimStart');
    check((s+s).trimEnd(), '', 'all whitespace trimEnd');
    check((s+text+s).trimStart(), text+s, 'leading Unicode whitespace');
    check((s+text+s).trimEnd(), s+text, 'trailing Unicode whitespace');
  }
  check(text.trim(), text, 'already trimmed string');
  check((' '+text).trimEnd(), ' '+text, 'trimEnd keeps leading whitespace');
  check((text+' ').trimStart(), text+' ', 'trimStart keeps trailing whitespace');
  check('\u200b'.trim(), '\u200b', 'zero width space is not whitespace');
  check(Number(' \ufeff-0\u2029'), -0, 'numeric trim');
  check(1/Number(' \ufeff-0\u2029'), -Infinity, 'numeric negative zero');
  check(parseFloat('  12.5tail'), 12.5, 'parseFloat validated span');
  check(JSON.stringify(text), '"ab\\ud800cd\\udc00"', 'JSON quote span copy');
  print('string-slice-trim:ok');
})();
