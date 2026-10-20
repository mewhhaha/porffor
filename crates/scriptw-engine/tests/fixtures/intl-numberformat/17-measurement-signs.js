function check(value, message) { if (!value) throw new Error(message); }
function same(actual, expected, message) { check(Object.is(actual, expected), message); }

for (var row of [['ja-JP', '時速', ' キロメートル'], ['ko-KR', '시속', '킬로미터']]) {
  var formatter = new Intl.NumberFormat(row[0], {
    style: 'unit', unit: 'kilometer-per-hour', unitDisplay: 'long'
  });
  for (var input of [[-987, '987'], [-0.001, '0.001'], [-0, '0']]) {
    var parts = formatter.formatToParts(input[0]);
    same(formatter.format(input[0]), row[1] + ' -' + input[1] + row[2], 'localized negative measurement');
    same(parts[0].type, 'unit', 'prefix remains a unit');
    same(parts[0].value, row[1], 'localized prefix');
    same(parts[1].type, 'literal', 'prefix spacing');
    same(parts[2].type, 'minusSign', 'sign follows measurement prefix');
    same(parts[3].type, 'integer', 'signed numeric placeholder');
  }
  var positive = new Intl.NumberFormat(row[0], {
    style: 'unit', unit: 'kilometer-per-hour', unitDisplay: 'long', signDisplay: 'always'
  });
  same(positive.format(987), row[1] + ' +987' + row[2], 'positive sign stays numeric');
}

var temperature = new Intl.NumberFormat('ja', {
  style: 'unit', unit: 'celsius', unitDisplay: 'long', maximumFractionDigits: 0
});
var rangeParts = temperature.formatRangeToParts(-3, -5);
var units = rangeParts.filter(function (part) { return part.type === 'unit'; });
same(units.length, 2, 'both unit affixes remain');
same(units[0].value, '摂氏', 'temperature prefix');
same(units[1].value, '度', 'temperature suffix');
same(units[0].source, 'shared', 'prefix is shared');
same(units[1].source, 'shared', 'suffix is shared');
var signs = rangeParts.filter(function (part) { return part.type === 'minusSign'; });
same(signs.length, 2, 'endpoint signs never collapse');
same(signs[0].source, 'startRange', 'first sign owner');
same(signs[1].source, 'endRange', 'second sign owner');
var approximate = temperature.formatRangeToParts(-2.9, -3.1);
same(approximate.map(function (part) { return part.type; }).join(','),
     'unit,literal,approximatelySign,minusSign,integer,literal,unit',
     'range approximation remains inside the measurement');

var dual = new Intl.NumberFormat('ar-u-nu-latn', {style:'unit', unit:'meter'});
var negativeDual = dual.formatToParts(-2);
same(negativeDual.filter(function (part) { return part.type === 'minusSign'; }).length,
     1, 'literal-only dual retains its negative sign');
same(negativeDual.filter(function (part) { return part.type === 'integer'; }).length,
     0, 'literal-only dual does not gain a numeric placeholder');
print('ok measurement signs');
