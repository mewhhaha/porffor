// General emitted-compiler controls. Sources and flags come from code-unit arrays.
function fromUnits(units) {
  var value = '';
  for (var i = 0; i < units.length; i++) value += String.fromCharCode(units[i]);
  return value;
}
function require(value, label) { if (!value) throw new Error(label); }
function check(test) {
  var expression = new RegExp(fromUnits(test.pattern), fromUnits(test.flags));
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
check({"name":"two hundred captures","pattern":[40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,40,104,101,108,108,111,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41],"flags":[],"input":[120,104,101,108,108,111],"captures":[[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111],[104,101,108,108,111]],"index":1,"indices":null});
check({"name":"two hundred noncaptures","pattern":[40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,40,63,58,104,101,108,108,111,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41,41],"flags":[],"input":[120,104,101,108,108,111],"captures":[[104,101,108,108,111]],"index":1,"indices":null});
check({"name":"mixed group numbering","pattern":[40,97,40,63,58,98,41,40,99,41,41],"flags":[],"input":[120,97,98,99,121],"captures":[[97,98,99],[97,98,99],[99]],"index":1,"indices":null});
check({"name":"ordered empty choice","pattern":[40,63,58,124,97,98,41,99],"flags":[],"input":[97,98,99],"captures":[[97,98,99]],"index":0,"indices":null});
check({"name":"ordered short alternative","pattern":[40,97,124,97,98,41,98],"flags":[],"input":[97,98,98],"captures":[[97,98],[97]],"index":0,"indices":null});
check({"name":"greedy capture","pattern":[40,97,43,41,97],"flags":[],"input":[97,97,97,97],"captures":[[97,97,97,97],[97,97,97]],"index":0,"indices":null});
check({"name":"lazy capture","pattern":[40,97,43,63,41,97],"flags":[],"input":[97,97,97,97],"captures":[[97,97],[97]],"index":0,"indices":null});
check({"name":"finite capture repetition","pattern":[40,97,98,41,123,50,44,51,125],"flags":[],"input":[97,98,97,98,97,98,97,98],"captures":[[97,98,97,98,97,98],[97,98]],"index":0,"indices":null});
check({"name":"nullable progress","pattern":[40,97,63,41,42,98],"flags":[],"input":[97,97,97,98],"captures":[[97,97,97,98],[97]],"index":0,"indices":null});
check({"name":"numbered reference","pattern":[40,97,124,98,41,92,49],"flags":[],"input":[120,97,97,98,98,121],"captures":[[97,97],[97]],"index":1,"indices":null});
check({"name":"unmatched reference","pattern":[40,63,58,97,124,40,98,41,41,92,49,99],"flags":[],"input":[97,99],"captures":[[97,99],null],"index":0,"indices":null});
check({"name":"forward reference","pattern":[92,49,40,97,41],"flags":[],"input":[97],"captures":[[97],[97]],"index":0,"indices":null});
check({"name":"negative lookahead","pattern":[97,40,63,33,98,41,99],"flags":[],"input":[97,98,99,97,99],"captures":[[97,99]],"index":3,"indices":null});
check({"name":"positive assertion capture","pattern":[97,40,63,61,40,98,124,99,41,41,92,49],"flags":[],"input":[120,97,99,100],"captures":[[97,99],[99]],"index":1,"indices":null});
check({"name":"folded union ranges","pattern":[91,97,45,122,48,45,57,93,43],"flags":[105],"input":[33,65,98,57,63],"captures":[[65,98,57]],"index":1,"indices":null});
check({"name":"folded negation","pattern":[91,94,97,45,99,93,43],"flags":[105],"input":[97,68,101,66],"captures":[[68,101]],"index":1,"indices":null});
check({"name":"folded reference and boundaries","pattern":[92,98,40,92,119,43,41,92,115,43,92,49,92,98],"flags":[105],"input":[104,101,108,108,111,32,72,69,76,76,79],"captures":[[104,101,108,108,111,32,72,69,76,76,79],[104,101,108,108,111]],"index":0,"indices":null});
check({"name":"unmatched indices","pattern":[40,97,41,63,98],"flags":[100,103],"input":[98],"captures":[[98],null],"index":0,"indices":[[0,1],null]});
check({"name":"multiline dotAll","pattern":[94,97,46,98,36],"flags":[109,115],"input":[120,10,97,10,98,10,122],"captures":[[97,10,98]],"index":2,"indices":null});
check({"name":"escaped syntax","pattern":[92,40,92,41,92,91,92,93,92,43,92,63],"flags":[],"input":[120,40,41,91,93,43,63,121],"captures":[[40,41,91,93,43,63]],"index":1,"indices":null});
check({"name":"legacy decimal fallback","pattern":[92,49,52,49,92,56],"flags":[],"input":[97,56],"captures":[[97,56]],"index":0,"indices":null});
check({"name":"omitted capture instructions","pattern":[40,97,40,98,40,99,41,41,41,123,48,125],"flags":[100],"input":[],"captures":[[],null,null,null],"index":0,"indices":[[0,0],null,null,null]});
check({"name":"legacy fold non ASCII range","pattern":[91,192,45,214,93,43],"flags":[105],"input":[224,214],"captures":[[224,214]],"index":0,"indices":null});
check({"name":"legacy long s restriction","pattern":[115],"flags":[105],"input":[383],"captures":null,"index":0,"indices":null});
check({"name":"legacy kelvin restriction","pattern":[107],"flags":[105],"input":[8490],"captures":null,"index":0,"indices":null});
check({"name":"legacy surrogate atom quantifier","pattern":[55357,57314,43],"flags":[],"input":[55357,57314,55357,57314],"captures":[[55357,57314]],"index":0,"indices":null});
check({"name":"legacy grouped surrogate pair","pattern":[40,63,58,55357,57314,41,43],"flags":[],"input":[55357,57314,55357,57314],"captures":[[55357,57314,55357,57314]],"index":0,"indices":null});
check({"name":"lone surrogate source","pattern":[55296],"flags":[],"input":[120,55296,121],"captures":[[55296]],"index":1,"indices":null});
function rejects(test) {
  var caught = false;
  try { new RegExp(fromUnits(test.pattern), fromUnits(test.flags)); }
  catch (error) { caught = error instanceof SyntaxError; }
  require(caught, test.name);
}
rejects({"name":"unclosed group","pattern":[40],"flags":[]});
rejects({"name":"stray close","pattern":[41],"flags":[]});
rejects({"name":"unclosed class","pattern":[91],"flags":[]});
rejects({"name":"reverse range","pattern":[91,122,45,97,93],"flags":[]});
rejects({"name":"no quantifier atom","pattern":[42],"flags":[]});
rejects({"name":"duplicate quantifier","pattern":[97,42,42],"flags":[]});
rejects({"name":"reverse repetition","pattern":[97,123,50,44,49,125],"flags":[]});
rejects({"name":"trailing escape","pattern":[92],"flags":[]});
rejects({"name":"duplicate flag","pattern":[97],"flags":[103,103]});
var order = [];
var source = { toString: function() { order.push(1); return fromUnits([40,97,41,43]); } };
var flags = { toString: function() { order.push(2); return fromUnits([105]); } };
var coerced = new RegExp(source, flags);
require(order.length === 2 && order[0] === 1 && order[1] === 2 && coerced.test(fromUnits([65])), 'coercion order');
var sentinel = {};
var threw = false;
try { new RegExp({ toString: function() { throw sentinel; } }, flags); }
catch (error) { threw = error === sentinel; }
require(threw && order.length === 2, 'coercion identity');
var originalSource = fromUnits([40,111,108,100,41]);
var replacementSource = fromUnits([40,110,101,119,41]);
var original = new RegExp(originalSource);
original.lastIndex = 7;
var rejected = false;
try { original.compile(fromUnits([40])); }
catch (error) { rejected = error instanceof SyntaxError; }
require(rejected && original.source === originalSource && original.lastIndex === 7, 'failed compile state');
require(original.exec(fromUnits([111,108,100]))[1] === fromUnits([111,108,100]), 'old program retained');
Object.defineProperty(original, 'lastIndex', { writable: false });
var writeFailed = false;
try { original.compile(replacementSource); }
catch (error) { writeFailed = error instanceof TypeError; }
require(writeFailed && original.source === replacementSource && original.lastIndex === 7, 'successful compile publication');
require(original.exec(fromUnits([110,101,119]))[1] === fromUnits([110,101,119]), 'replacement program retained');
var shared = new RegExp(fromUnits([40,97,124,98,41,43]), fromUnits([100,103]));
var clone = new RegExp(shared);
var first = shared.exec(fromUnits([97,98]));
var other = new RegExp(fromUnits([40,120,41,43]));
for (var churn = 0; churn < 8; churn++) require(other.exec(fromUnits([120,120]))[1] === fromUnits([120]), 'scratch churn');
shared.lastIndex = 1;
require(clone.lastIndex === 0, 'independent clone state');
var clonedMatch = clone.exec(fromUnits([98,97]));
require(first[0] === fromUnits([97,98]) && first[1] === fromUnits([98]) && first.indices[0][1] === 2, 'first retained result');
require(clonedMatch[0] === fromUnits([98,97]) && clonedMatch[1] === fromUnits([97]) && clone !== shared, 'clone program lifetime');
require(shared.lastIndex === 1 && clone.lastIndex === 2, 'separate lastIndex');
check({"name":"empty complement class","pattern":[91,94,93],"flags":[],"input":[10],"captures":[[10]],"index":0,"indices":null});
check({"name":"empty class","pattern":[91,93],"flags":[],"input":[97],"captures":null,"index":0,"indices":null});
check({"name":"class backspace","pattern":[91,92,98,93],"flags":[],"input":[8],"captures":[[8]],"index":0,"indices":null});
check({"name":"legacy class control underscore","pattern":[91,92,99,95,93],"flags":[],"input":[31],"captures":[[31]],"index":0,"indices":null});
check({"name":"legacy class control digit","pattern":[91,92,99,48,93],"flags":[],"input":[16],"captures":[[16]],"index":0,"indices":null});
check({"name":"control letter","pattern":[92,99,65],"flags":[],"input":[1],"captures":[[1]],"index":0,"indices":null});
check({"name":"legacy class set range right","pattern":[91,97,45,92,100,93,43],"flags":[],"input":[97,45,53],"captures":[[97,45,53]],"index":0,"indices":null});
check({"name":"legacy class set range left","pattern":[91,92,100,45,97,93,43],"flags":[],"input":[53,45,97],"captures":[[53,45,97]],"index":0,"indices":null});
check({"name":"incomplete hexadecimal identity","pattern":[92,120,90],"flags":[],"input":[120,90],"captures":[[120,90]],"index":0,"indices":null});
check({"name":"incomplete unicode identity","pattern":[92,117,90,90,90,90],"flags":[],"input":[117,90,90,90,90],"captures":[[117,90,90,90,90]],"index":0,"indices":null});
check({"name":"legacy unicode brace is repetition","pattern":[92,117,123,51,125],"flags":[],"input":[117,117,117],"captures":[[117,117,117]],"index":0,"indices":null});
check({"name":"legacy property lead is identity","pattern":[92,112,123,50,125],"flags":[],"input":[112,112],"captures":[[112,112]],"index":0,"indices":null});
check({"name":"legacy named reference lead is identity","pattern":[92,107,60,110,97,109,101,62],"flags":[],"input":[107,60,110,97,109,101,62],"captures":[[107,60,110,97,109,101,62]],"index":0,"indices":null});
check({"name":"escaped nonascii identity","pattern":[92,233,43],"flags":[],"input":[233,233],"captures":[[233,233]],"index":0,"indices":null});
check({"name":"legacy two digit octal boundary","pattern":[92,55,55,55],"flags":[],"input":[63,55],"captures":[[63,55]],"index":0,"indices":null});
check({"name":"legacy literal close delimiters","pattern":[93,125],"flags":[],"input":[120,93,125],"captures":[[93,125]],"index":1,"indices":null});
check({"name":"legacy malformed braces literal","pattern":[97,123,120,125],"flags":[],"input":[97,123,120,125],"captures":[[97,123,120,125]],"index":0,"indices":null});
check({"name":"lookahead zero quantifier","pattern":[40,63,61,40,97,41,41,123,48,125,98],"flags":[],"input":[98],"captures":[[98],null],"index":0,"indices":null});
check({"name":"lookahead optional progress","pattern":[40,63,61,40,97,41,41,42,97],"flags":[],"input":[97],"captures":[[97],null],"index":0,"indices":null});
check({"name":"negative lookahead optional progress","pattern":[40,63,33,40,97,41,41,42,97],"flags":[],"input":[97],"captures":[[97],null],"index":0,"indices":null});
check({"name":"optional unmatched reference progress","pattern":[40,97,41,63,92,49,42],"flags":[],"input":[],"captures":[[],null],"index":0,"indices":null});
check({"name":"alternative unmatched reference progress","pattern":[40,63,58,40,97,41,124,98,41,92,49,42],"flags":[],"input":[98],"captures":[[98],null],"index":0,"indices":null});
check({"name":"nested nullable lazy capture","pattern":[40,97,63,98,63,63,41,42],"flags":[],"input":[97,98],"captures":[[97,98],[98]],"index":0,"indices":null});
check({"name":"nullable greedy before suffix","pattern":[40,97,63,41,42,98],"flags":[],"input":[97,97,98],"captures":[[97,97,98],[97]],"index":0,"indices":null});
check({"name":"nullable lazy before suffix","pattern":[40,97,63,41,42,63,98],"flags":[],"input":[97,97,98],"captures":[[97,97,98],[97]],"index":0,"indices":null});
check({"name":"nested empty groups","pattern":[40,63,58,40,63,58,41,41,42],"flags":[],"input":[],"captures":[[]],"index":0,"indices":null});
var resourceExpression = new RegExp(fromUnits([40,97,41]));
resourceExpression.lastIndex = 4;
var exhausted = false;
try { resourceExpression.compile(fromUnits([97,123,53,48,48,48,125])); }
catch (error) { exhausted = error instanceof RangeError; }
require(exhausted && resourceExpression.source === fromUnits([40,97,41]) && resourceExpression.lastIndex === 4, 'resource recompile rollback');
require(resourceExpression.exec(fromUnits([97]))[1] === fromUnits([97]), 'resource failure retains old program');
check({"name":"incomplete control keeps backslash","pattern":[92,99],"flags":[],"input":[92,99],"captures":[[92,99]],"index":0,"indices":null});
check({"name":"incomplete control rejects marker alone","pattern":[92,99],"flags":[],"input":[99],"captures":null,"index":0,"indices":null});
check({"name":"digit control outside class","pattern":[92,99,48],"flags":[],"input":[92,99,48],"captures":[[92,99,48]],"index":0,"indices":null});
check({"name":"underscore control outside class","pattern":[92,99,95],"flags":[],"input":[92,99,95],"captures":[[92,99,95]],"index":0,"indices":null});
check({"name":"incomplete control marker quantifier","pattern":[92,99,43],"flags":[],"input":[92,99,99,99],"captures":[[92,99,99,99]],"index":0,"indices":null});
// Patterns assembled from independent grammar pieces exercise a broad computed composition.
  var alpha = "[a-z]",
    digit = "[0-9]",
    alphanum = "[a-z0-9]",
    variant = "(" + alphanum + "{5,8}|(?:" + digit + alphanum + "{3}))",
    region = "(" + alpha + "{2}|" + digit + "{3})",
    script = "(" + alpha + "{4})",
    language = "(" + alpha + "{2,3}|" + alpha + "{5,8})",
    privateuse = "(x(-[a-z0-9]{1,8})+)",
    singleton = "(" + digit + "|[a-wy-z])",
    attribute= "(" + alphanum + "{3,8})",
    keyword = "(" + alphanum + alpha + "(-" + alphanum + "{3,8})*)",
    unicode_locale_extensions = "(u((-" + keyword + ")+|((-" + attribute + ")+(-" + keyword + ")*)))",
    tlang = "(" + language + "(-" + script + ")?(-" + region + ")?(-" + variant + ")*)",
    tfield = "(" + alpha + digit + "(-" + alphanum + "{3,8})+)",
    transformed_extensions = "(t((-" + tlang + "(-" + tfield + ")*)|(-" + tfield + ")+))",
    other_singleton = "(" + digit + "|[a-sv-wy-z])",
    other_extensions = "(" + other_singleton + "(-" + alphanum + "{2,8})+)",
    extension = "(" + unicode_locale_extensions + "|" + transformed_extensions + "|" + other_extensions + ")",
    locale_id = language + "(-" + script + ")?(-" + region + ")?(-" + variant + ")*(-" + extension + ")*(-" + privateuse + ")?",
    languageTag = "^(" + locale_id + ")$",
    languageTagRE = new RegExp(languageTag, "i");

  var duplicateSingleton = "-" + singleton + "-(.*-)?\\1(?!" + alphanum + ")",
    duplicateSingletonRE = new RegExp(duplicateSingleton, "i"),
    duplicateVariant = "(" + alphanum + "{2,8}-)+" + variant + "-(" + alphanum + "{2,8}-)*\\2(?!" + alphanum + ")",
    duplicateVariantRE = new RegExp(duplicateVariant, "i");

  var transformKeyRE = new RegExp("^" + alpha + digit + "$", "i");
if (!languageTagRE.test("en-US") || languageTagRE.test("")) throw "language grammar";
require(languageTagRE.test('en-Latn-US-u-ca-gregory'), 'composed language extension');
require(duplicateSingletonRE.test('en-a-abc-a-def') && !duplicateSingletonRE.test('en-a-abc-b-def'), 'computed duplicate singleton');
require(duplicateVariantRE.test('en-abcde-abcde') && !duplicateVariantRE.test('en-abcde-fghij'), 'computed duplicate variant');
require(transformKeyRE.test('a0') && !transformKeyRE.test('aa'), 'computed transform key');
true;
