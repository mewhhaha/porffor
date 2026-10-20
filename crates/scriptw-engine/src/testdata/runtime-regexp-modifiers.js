// Pattern sources and flags use code units so no literal cache row can satisfy them.
function fromUnits(units) {
  var value = '';
  for (var i = 0; i < units.length; i++) value += String.fromCharCode(units[i]);
  return value;
}
function require(value, label) { if (!value) throw new Error(label); }
function check(test) {
  var flags = fromUnits(test.flags);
  var expression = new RegExp(fromUnits(test.pattern), flags);
  require(expression.ignoreCase === (flags.indexOf('i') >= 0), test.name);
  require(expression.multiline === (flags.indexOf('m') >= 0), test.name);
  require(expression.dotAll === (flags.indexOf('s') >= 0), test.name);
  require(expression.flags === flags, test.name);
  var input = fromUnits(test.input);
  var match = expression.exec(input);
  if (test.captures === null) {
    require(match === null, test.name);
    return;
  }
  require(match !== null && match.length === test.captures.length, test.name);
  require(match.index === test.index && match.input === input, test.name);
  for (var i = 0; i < test.captures.length; i++) {
    var expected = test.captures[i] === null ? undefined : fromUnits(test.captures[i]);
    require(match[i] === expected, test.name);
  }
  if (test.indices !== null) {
    require(match.indices.length === test.indices.length, test.name);
    for (var i = 0; i < test.indices.length; i++) {
      var pair = test.indices[i];
      require(pair === null ? match.indices[i] === undefined :
        match.indices[i][0] === pair[0] && match.indices[i][1] === pair[1], test.name);
    }
  }
}
check({"name":"add case within scope","pattern":[94,40,63,105,58,97,66,41,99,36],"flags":[],"input":[65,98,99],"captures":[[65,98,99]],"index":0,"indices":null});
check({"name":"restore case after scope","pattern":[94,40,63,105,58,97,66,41,99,36],"flags":[],"input":[65,66,67],"captures":null,"index":0,"indices":null});
check({"name":"remove case within scope","pattern":[94,40,63,45,105,58,97,66,41,99,36],"flags":[105],"input":[97,66,67],"captures":[[97,66,67]],"index":0,"indices":null});
check({"name":"removed case rejects folding","pattern":[94,40,63,45,105,58,97,66,41,99,36],"flags":[105],"input":[65,98,99],"captures":null,"index":0,"indices":null});
check({"name":"nested case restoration","pattern":[94,40,63,105,58,97,40,63,45,105,58,98,41,99,41,100,36],"flags":[],"input":[65,98,67,100],"captures":[[65,98,67,100]],"index":0,"indices":null});
check({"name":"nested case removal","pattern":[94,40,63,105,58,97,40,63,45,105,58,98,41,99,41,100,36],"flags":[],"input":[65,66,67,100],"captures":null,"index":0,"indices":null});
check({"name":"root case restoration","pattern":[94,40,63,105,58,97,40,63,45,105,58,98,41,99,41,100,36],"flags":[],"input":[65,98,67,68],"captures":null,"index":0,"indices":null});
check({"name":"ordinary groups inherit","pattern":[94,40,63,105,58,40,97,41,40,63,58,98,41,40,63,61,99,41,99,41,100,36],"flags":[],"input":[65,66,67,100],"captures":[[65,66,67,100],[65]],"index":0,"indices":null});
check({"name":"capturing group keeps modifier owner","pattern":[94,40,63,105,58,40,97,40,63,45,105,58,98,41,99,41,41,100,36],"flags":[],"input":[65,98,67,100],"captures":[[65,98,67,100],[65,98,67]],"index":0,"indices":null});
check({"name":"remove then add then restore","pattern":[94,40,63,45,105,58,97,40,63,105,58,98,41,99,41,100,36],"flags":[105],"input":[97,66,99,68],"captures":[[97,66,99,68]],"index":0,"indices":null});
check({"name":"outer removal persists","pattern":[94,40,63,45,105,58,97,40,63,105,58,98,41,99,41,100,36],"flags":[105],"input":[65,66,99,68],"captures":null,"index":0,"indices":null});
check({"name":"first alternative scope","pattern":[94,40,63,58,40,63,105,58,97,41,98,124,99,40,63,105,58,100,41,41,101,36],"flags":[],"input":[65,98,101],"captures":[[65,98,101]],"index":0,"indices":null});
check({"name":"second alternative scope","pattern":[94,40,63,58,40,63,105,58,97,41,98,124,99,40,63,105,58,100,41,41,101,36],"flags":[],"input":[99,68,101],"captures":[[99,68,101]],"index":0,"indices":null});
check({"name":"alternatives do not leak","pattern":[94,40,63,58,40,63,105,58,97,41,98,124,99,40,63,105,58,100,41,41,101,36],"flags":[],"input":[67,68,101],"captures":null,"index":0,"indices":null});
check({"name":"inner alternative restoration","pattern":[94,40,63,105,58,97,124,40,63,45,105,58,98,41,124,99,41,100,36],"flags":[],"input":[67,100],"captures":[[67,100]],"index":0,"indices":null});
check({"name":"inner alternative removal","pattern":[94,40,63,105,58,97,124,40,63,45,105,58,98,41,124,99,41,100,36],"flags":[],"input":[66,100],"captures":null,"index":0,"indices":null});
check({"name":"quantified capture indices","pattern":[94,40,63,105,58,40,97,124,98,41,41,43,99,36],"flags":[100],"input":[65,66,98,99],"captures":[[65,66,98,99],[98]],"index":0,"indices":[[0,4],[2,3]]});
check({"name":"nullable quantified capture rollback","pattern":[94,40,63,105,58,40,97,41,63,41,42,98,36],"flags":[],"input":[65,65,98],"captures":[[65,65,98],[65]],"index":0,"indices":null});
check({"name":"nullable unmatched reference progress","pattern":[94,40,63,105,58,40,97,41,63,92,49,42,41,98,36],"flags":[],"input":[98],"captures":[[98],null],"index":0,"indices":null});
check({"name":"folded class and restored class","pattern":[94,40,63,105,58,91,97,45,99,93,41,40,63,45,105,58,91,97,45,99,93,41,36],"flags":[],"input":[66,99],"captures":[[66,99]],"index":0,"indices":null});
check({"name":"class removal","pattern":[94,40,63,105,58,91,97,45,99,93,41,40,63,45,105,58,91,97,45,99,93,41,36],"flags":[],"input":[66,67],"captures":null,"index":0,"indices":null});
check({"name":"negation follows case closure","pattern":[94,40,63,105,58,91,94,97,93,41,36],"flags":[],"input":[65],"captures":null,"index":0,"indices":null});
check({"name":"negated folded class match","pattern":[94,40,63,105,58,91,94,97,93,41,36],"flags":[],"input":[66],"captures":[[66]],"index":0,"indices":null});
check({"name":"legacy nonascii case closure","pattern":[94,40,63,105,58,92,117,48,49,48,48,41,36],"flags":[],"input":[257],"captures":[[257]],"index":0,"indices":null});
check({"name":"legacy Kelvin restriction","pattern":[94,40,63,105,58,107,41,36],"flags":[],"input":[8490],"captures":null,"index":0,"indices":null});
check({"name":"reference site case override","pattern":[94,40,97,41,40,63,105,58,92,49,41,36],"flags":[],"input":[97,65],"captures":[[97,65],[97]],"index":0,"indices":null});
check({"name":"reference outside scope is sensitive","pattern":[94,40,63,105,58,40,97,41,41,92,49,36],"flags":[],"input":[65,97],"captures":null,"index":0,"indices":null});
check({"name":"reference outside scope exact match","pattern":[94,40,63,105,58,40,97,41,41,92,49,36],"flags":[],"input":[65,65],"captures":[[65,65],[65]],"index":0,"indices":null});
check({"name":"reference removes global case","pattern":[94,40,97,41,40,63,45,105,58,92,49,41,36],"flags":[105],"input":[97,65],"captures":null,"index":0,"indices":null});
check({"name":"positive lookahead inherits","pattern":[94,40,63,105,58,40,63,61,40,97,41,41,92,49,41,98,36],"flags":[],"input":[65,98],"captures":[[65,98],[65]],"index":0,"indices":null});
check({"name":"lookahead scope does not leak","pattern":[94,40,63,61,40,63,105,58,97,41,41,97,36],"flags":[],"input":[65],"captures":null,"index":0,"indices":null});
check({"name":"negative lookahead inherits","pattern":[94,40,63,105,58,40,63,33,97,41,98,41,99,36],"flags":[],"input":[66,99],"captures":[[66,99]],"index":0,"indices":null});
check({"name":"negative lookahead rejects folded literal","pattern":[94,40,63,105,58,40,63,33,97,41,98,41,99,36],"flags":[],"input":[65,99],"captures":null,"index":0,"indices":null});
check({"name":"add dotAll","pattern":[94,40,63,115,58,46,41,46,36],"flags":[],"input":[10,120],"captures":[[10,120]],"index":0,"indices":null});
check({"name":"restore dotAll","pattern":[94,40,63,115,58,46,41,46,36],"flags":[],"input":[10,10],"captures":null,"index":0,"indices":null});
check({"name":"remove dotAll","pattern":[94,40,63,45,115,58,46,41,46,36],"flags":[115],"input":[10,120],"captures":null,"index":0,"indices":null});
check({"name":"restore global dotAll","pattern":[94,40,63,45,115,58,46,41,46,36],"flags":[115],"input":[120,10],"captures":[[120,10]],"index":0,"indices":null});
check({"name":"nested dotAll restoration","pattern":[94,40,63,115,58,46,40,63,45,115,58,46,41,46,41,46,36],"flags":[],"input":[10,120,10,121],"captures":[[10,120,10,121]],"index":0,"indices":null});
check({"name":"nested dotAll removal","pattern":[94,40,63,115,58,46,40,63,45,115,58,46,41,46,41,46,36],"flags":[],"input":[10,10,10,121],"captures":null,"index":0,"indices":null});
check({"name":"dotAll inherited by ordinary groups","pattern":[94,40,63,115,58,40,46,41,40,63,58,46,41,40,63,61,46,41,40,63,33,120,41,46,41,46,36],"flags":[],"input":[10,10,10,121],"captures":[[10,10,10,121],[10]],"index":0,"indices":null});
check({"name":"add multiline","pattern":[40,63,109,58,94,97,36,41],"flags":[],"input":[10,97,10],"captures":[[97]],"index":1,"indices":null});
check({"name":"remove multiline","pattern":[40,63,45,109,58,94,97,36,41],"flags":[109],"input":[10,97,10],"captures":null,"index":0,"indices":null});
check({"name":"nested multiline restoration","pattern":[40,63,109,58,94,97,36,92,110,40,63,45,109,58,98,41,92,110,94,99,36,41],"flags":[],"input":[97,10,98,10,99],"captures":[[97,10,98,10,99]],"index":0,"indices":null});
check({"name":"nested multiline removal","pattern":[40,63,109,58,94,97,36,92,110,40,63,45,109,58,94,98,36,41,92,110,94,99,36,41],"flags":[],"input":[97,10,98,10,99],"captures":null,"index":0,"indices":null});
check({"name":"root multiline restoration","pattern":[40,63,109,58,94,97,36,41,92,110,94,98,36],"flags":[],"input":[97,10,98],"captures":null,"index":0,"indices":null});
check({"name":"global multiline restoration","pattern":[40,63,45,109,58,97,41,92,110,94,98,36],"flags":[109],"input":[97,10,98],"captures":[[97,10,98]],"index":0,"indices":null});
check({"name":"mixed add and remove flags","pattern":[40,63,105,109,45,115,58,94,97,46,36,41],"flags":[115],"input":[10,65,98,10],"captures":[[65,98]],"index":1,"indices":null});
check({"name":"remove every global flag","pattern":[40,63,45,105,109,115,58,94,97,46,36,41],"flags":[105,109,115],"input":[97,98],"captures":[[97,98]],"index":0,"indices":null});
check({"name":"removed dotAll stays off with other removals","pattern":[40,63,45,105,109,115,58,94,97,46,36,41],"flags":[105,109,115],"input":[97,10],"captures":null,"index":0,"indices":null});
check({"name":"empty remove modifier list","pattern":[40,63,105,45,58,97,41],"flags":[],"input":[65],"captures":[[65]],"index":0,"indices":null});
check({"name":"state free scoped group can erase huge bounds","pattern":[40,63,105,109,115,58,41,123,57,57,57,57,57,57,57,57,57,57,57,57,57,57,57,57,57,57,57,57,57,57,57,57,125],"flags":[],"input":[120],"captures":[[]],"index":0,"indices":null});

check({"name":"finite lazy scoped captures","pattern":[94,40,63,105,58,40,97,41,41,123,49,44,51,125,63,98,36],"flags":[100],"input":[65,65,98],"captures":[[65,65,98],[65]],"index":0,"indices":[[0,3],[1,2]]});

function invalid(pattern) {
  var caught;
  try { new RegExp(fromUnits(pattern)); } catch (error) { caught = error; }
  require(caught instanceof SyntaxError, 'invalid modifier prefix must throw SyntaxError');
}
invalid([40,63,45,58,97,41]);
invalid([40,63,105,105,58,97,41]);
invalid([40,63,109,109,58,97,41]);
invalid([40,63,115,115,58,97,41]);
invalid([40,63,45,105,105,58,97,41]);
invalid([40,63,105,45,105,58,97,41]);
invalid([40,63,105,109,45,109,115,58,97,41]);
invalid([40,63,105,115,45,115,58,97,41]);
invalid([40,63,105,45,45,115,58,97,41]);
invalid([40,63,45,45,105,58,97,41]);
invalid([40,63,103,58,97,41]);
invalid([40,63,100,58,97,41]);
invalid([40,63,117,58,97,41]);
invalid([40,63,118,58,97,41]);
invalid([40,63,121,58,97,41]);
invalid([40,63,105,88,58,97,41]);
invalid([40,63,32,105,58,97,41]);
invalid([40,63,105,61,97,41]);
invalid([40,63,105]);
invalid([40,63,105,45]);
invalid([40,63,105,58,97]);
check({"name":"add case within scope","pattern":[94,40,63,105,58,97,66,41,99,36],"flags":[],"input":[65,98,99],"captures":[[65,98,99]],"index":0,"indices":null});
true;
