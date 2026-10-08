for (var mode of [117, 118]) {
  for (var row of [
    ['ASCII_Hex_Digit', 'AHex', 0x46, 0x47],
    ['ID_Start', 'IDS', 0x61, 0x30],
    ['ID_Continue', 'IDC', 0x30, 0x2d],
    ['IDS_Binary_Operator', 'IDSB', 0x2ff0, 0x61],
    ['IDS_Trinary_Operator', 'IDST', 0x2ff2, 0x61],
    ['White_Space', 'space', 0xa0, 0x61]
  ]) {
    for (var name of [row[0], row[1]]) {
      var expression = property(name, 112, [mode]);
      require(expression.test(codePoint(row[2])) && !expression.test(codePoint(row[3])), 'exact binary aliases retain membership');
    }
  }
  for (var name of ['General_Category=Uppercase_Letter', 'gc=Lu', 'Uppercase_Letter', 'Lu']) {
    var category = property(name, 112, [mode]);
    require(category.test('A') && !category.test('a'), 'exact GC aliases and lone GC values');
  }
  for (var name of ['Script=Greek', 'sc=Grek']) {
    var greek = property(name, 112, [mode]);
    require(greek.test(codePoint(0x3a9)) && !greek.test('A'), 'exact Script family and value aliases');
  }
  for (var name of ['Script_Extensions=Hiragana', 'scx=Hira']) {
    require(property(name, 112, [mode]).test(codePoint(0x30fc)), 'Script_Extensions includes shared Hiragana/Katakana mark');
  }
  require(!property('Script=Hiragana', 112, [mode]).test(codePoint(0x30fc)), 'Script is distinct from Script_Extensions');
  require(property('sc=Zyyy', 112, [mode]).test(codePoint(0x30fc)), 'shared mark has Script Common');
  for (var script of [
    ['Beria_Erfe', 'Berf', 0x16ea0, 0x16eb9],
    ['Sidetic', 'Sidt', 0x10940, 0x1095a],
    ['Tai_Yo', 'Tayo', 0x1e6c0, 0x1e6df],
    ['Tolong_Siki', 'Tols', 0x11db0, 0x11ddc]
  ]) {
    var member = codePoint(script[2]);
    var hole = codePoint(script[3]);
    for (var name of ['Script=' + script[0], 'sc=' + script[1], 'Script_Extensions=' + script[0], 'scx=' + script[1]]) {
      var expression = property(name, 112, [mode]);
      require(expression.test(member), 'Unicode17 long/short Script values in both families');
      require(!expression.test(hole) && !expression.test('A'), 'Unicode17 Script range holes and nonmembers');
    }
    var complement = property('sc=' + script[1], 80, [mode]);
    require(!complement.test(member) && complement.test(hole), 'Unicode17 Script complement uses the complete code-point domain');
    require(property('Assigned', 112, [mode]).test(member), 'Assigned includes Unicode17 additions');
  }
  var maximum = codePoint(0x10ffff);
  var lone = codePoint(0xd800);
  var face = codePoint(0x1f600);
  var any = property('Any', 112, [mode]);
  require(any.test(maximum) && any.test(lone) && any.test(face), 'Any covers maximum, lone surrogate and paired astral character');
  require(!property('Any', 80, [mode]).test(maximum), 'P Any is empty');
  var surrogate = property('gc=Cs', 112, [mode]);
  require(surrogate.test(lone) && !surrogate.test(face), 'surrogate property distinguishes lone unit and scalar pair');
  var assigned = property('Assigned', 112, [mode]);
  require(assigned.test(lone) && !assigned.test(maximum), 'Assigned follows General_Category rather than scalar-only encoding');
  require(property('NChar', 112, [mode]).test(maximum), 'maximum is a noncharacter');
  var indexed = property('Any', 112, [100, 103, mode]);
  var match = indexed.exec(face);
  require(match && match[0] === face && match.indices[0][1] === 2 && indexed.lastIndex === 2, 'property matching publishes UTF16 indices and advancement');
}
print('regexp-property-domains:ok');
262;
