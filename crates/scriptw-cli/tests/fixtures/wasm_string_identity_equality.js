(function () {
  function check(value, expected, name) { if (value !== expected) throw new Error(name); }
  var text = 'a\ud800\u00e9';
  var same = text;
  var distinct = ('z' + text).slice(1);
  check(text === same, true, 'same UTF16 value');
  check(text === distinct, true, 'equal content from distinct construction');
  check(text === 'a\ud800\u00e8', false, 'unequal nonASCII content');
  var object = {};
  object[text] = 17;
  check(object[same], 17, 'same key');
  check(object[distinct], 17, 'distinct equal key');
  object['a\ud800\u00e8'] = 19;
  check(object[text], 17, 'near-miss keys separate');
  check(object['a\ud800\u00e8'], 19, 'near-miss lookup');
  var map = new Map([[text, 23]]);
  check(map.get(distinct), 23, 'Map equal-content key');
  var set = new Set([text, same, distinct]);
  check(set.size, 1, 'Set equal-content deduplication');
  check(/abc/i.test('AbC'), true, 'ASCII folding');
  check(/abc/i.test('AbD'), false, 'ASCII folding mismatch');
  check(/\u00e9/i.test('\u00e9'), true, 'same nonASCII regexp');
  print('string-identity-equality:ok');
})();
