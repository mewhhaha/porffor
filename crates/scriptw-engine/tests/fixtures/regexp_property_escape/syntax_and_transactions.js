for (var mode of [117, 118]) {
  for (var invalid of [
    'Id_Start', 'Id_Continue', 'Ids_Binary_Operator', 'Ids_Trinary_Operator',
    'ascii', 'Alphabeticx', 'No_Such_Property', 'IsGreek',
    'general_category=Lu', 'GC=Lu', 'gc=lu', 'gc=Alpha',
    'Script=Letter', 'sc=Greekx', 'sc=greek', 'Sc=Grek',
    'script_extensions=Hira', 'Scx=Hira', 'scx=Uppercase_Letter',
    'sc=Assigned', 'gc=Beria_Erfe', 'sc=Beria_erfe', 'scx=berf',
    'ASCII=Yes', 'Alphabetic=True', 'General_Category=', '=Lu',
    'gc=Lu=Ll', 'Old Persian', 'sc=Old-Persian', '', 'ASCII!', 'ASCI I'
  ]) {
    rejects(propertyUnits(invalid, 112), [mode]);
  }
  for (var invalidClass of ['Id_Start', 'Id_Continue', 'Ids_Binary_Operator', 'Ids_Trinary_Operator', 'Script=Letter', 'gc=Alpha', 'Scx=Hira', 'ASCII=Yes']) {
    rejects([91].concat(propertyUnits(invalidClass, 80), [93]), [mode]);
  }
  for (var malformed of [
    [92, 112], [92, 80], [92, 112, 65], [92, 112, 123],
    [92, 112, 123, 65, 83, 67, 73, 73],
    [92, 112, 123, 92, 117, 54, 49, 125],
    [92, 112, 123, 0xe9, 125]
  ]) rejects(malformed, [mode]);
  for (var endpoint of [
    [91].concat(propertyUnits('ASCII', 112), [45, 122, 93]),
    [91, 97, 45].concat(propertyUnits('ASCII', 112), [93]),
    [91].concat(propertyUnits('ASCII', 80), [45, 122, 93]),
    [91, 97, 45].concat(propertyUnits('ASCII', 80), [93])
  ]) rejects(endpoint, [mode]);
  var repeated = computed([94, 91].concat(propertyUnits('ASCII', 112), propertyUnits('ASCII', 112), propertyUnits('gc=Lu', 112), [93, 43, 36]), [mode]);
  require(repeated.test('A' + codePoint(0x3a9)) && !repeated.test(codePoint(0x3c0)), 'repeated property operands preserve the class union');
  var ascii = property('ASCII', 112, [mode]);
  var greek = property('sc=Grek', 112, [mode]);
  var notAscii = property('ASCII', 80, [mode]);
  require(ascii.test('A') && !ascii.test(codePoint(0x3a9)) && greek.test(codePoint(0x3a9)) && notAscii.test(codePoint(0x3a9)), 'later descriptors do not mutate earlier property bitmaps');
  var namedUnits = [94, 40, 63, 60, 110, 62].concat(propertyUnits('ASCII', 112), [43, 41, 92, 107, 60, 110, 62, 36]);
  var named = computed(namedUnits, [100, mode]);
  var namedMatch = named.exec('abab');
  require(namedMatch && namedMatch.groups.n === 'ab' && namedMatch.indices.groups.n === namedMatch.indices[1], 'named captures and backreferences retain property atoms');
  var numbered = computed([94, 40].concat(propertyUnits('ASCII', 112), [41, 92, 49, 36]), [mode]);
  require(numbered.test('aa') && !numbered.test('ab'), 'numbered neighbors retain their completed capture inventory');
  var ordinary = computed([94, 97, 46, 98, 36], [mode]);
  require(ordinary.test('axb') && !ordinary.test('axc'), 'ordinary runtime atoms remain intact');
  var receiver = computed([111, 108, 100], [103]);
  receiver.lastIndex = 7;
  var failed;
  try { receiver.compile(fromUnits([91].concat(propertyUnits('ASCII', 112), [45, 122, 93])), fromUnits([mode])); }
  catch (error) { failed = error instanceof SyntaxError; }
  require(failed && receiver.source === 'old' && receiver.flags === 'g' && receiver.lastIndex === 7, 'invalid property range cannot publish a partial receiver');
  receiver.lastIndex = 0;
  require(receiver.test('old') && receiver.lastIndex === 3, 'old descriptor survives syntax rollback');
  receiver.compile(fromUnits(namedUnits), fromUnits([100, mode]));
  var installed = receiver.exec('abab');
  require(receiver.lastIndex === 0 && installed && installed.groups.n === 'ab', 'valid property recompilation publishes a complete recovered descriptor');
  var installedSource = receiver.source;
  receiver.lastIndex = 9;
  failed = false;
  try { receiver.compile(fromUnits(propertyUnits('Script=Letter', 112)), fromUnits([mode])); }
  catch (error) { failed = error instanceof SyntaxError; }
  require(failed && receiver.source === installedSource && receiver.lastIndex === 9 && receiver.flags === (mode === 117 ? 'du' : 'dv'), 'cross-domain recompile preserves the new descriptor and public slots');
  require(receiver.exec('abab').groups.n === 'ab', 'cross-domain rollback also preserves private named descriptor membership');
}
require(computed(propertyUnits('ASCII', 112), []).test('p{ASCII}'), 'Legacy identity escape path remains separate');
print('regexp-property-syntax:ok');
262;
