function check(value, label) { if (!value) throw new Error(label); }
var parsers = [parseInt, Number.parseInt];
for (var p = 0; p < parsers.length; p++) {
  var parse = parsers[p], log = [];
  var radix = {
    get [Symbol.toPrimitive]() {
      log.push('get radix');
      return function(hint) { log.push('call radix ' + hint); return 4294967312.75; };
    }
  };
  var input = {
    get toString() {
      log.push('get string');
      return function() { log.push('call string'); return '10'; };
    }
  };
  check(parse(input, radix) === 16, 'coercive radix value');
  check(log.join('|') === 'get string|call string|get radix|call radix number',
        'ordered single conversions');
  log = [];
  radix = {valueOf() { throw new Error('stale radix conversion'); }};
  input = {toString() {
    log.push('string');
    radix.valueOf = function() { log.push('new radix'); return 1e308; };
    return '0x10';
  }};
  check(parse(input, radix) === 16 && log.join('|') === 'string|new radix',
        'string conversion mutation precedes radix conversion');
}
