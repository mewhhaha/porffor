// Dynamic Unicode syntax controls retain genuine SyntaxError outcomes.
// Clean runtime patterns and finite cached programs remain valid. Computed
// Unicode-only patterns beyond the runtime compiler are separate host-gap
// fixtures; their capability failure cannot be caught as a JavaScript error.
function checkThrows(source, flags, label) {
  var threw = false;
  try {
    RegExp(source, flags);
  } catch (error) {
    if (!(error instanceof SyntaxError)) throw label + ": wrong error";
    threw = true;
  }
  if (!threw) throw label + ": no throw";
}
function checkNoThrow(source, flags, label) {
  try {
    RegExp(source, flags);
  } catch (error) {
    throw label + ": threw " + error.constructor.name;
  }
}

var tail = String.fromCharCode(0x41);
checkThrows("\\" + tail, "u", "dynamic-identity-atom");
checkThrows("[\\" + tail + "]", "u", "dynamic-identity-class");
checkThrows("\\c", "u", "dynamic-control-bare");
checkThrows("\\c" + String.fromCharCode(0x30), "u", "dynamic-control-digit");
checkThrows("[\\B]", "u", "dynamic-class-B");
checkThrows("]", "u", "dynamic-bare-bracket");
checkThrows("\\" + String.fromCharCode(0x20), "u", "dynamic-identity-space");
checkNoThrow("\\cA", "u", "dynamic-control-letter");
checkNoThrow("\\/", "u", "dynamic-slash");
checkNoThrow("[\\-]", "u", "dynamic-class-dash");
checkNoThrow("a+", "u", "dynamic-simple");
checkNoThrow("\\p{ASCII}", "u", "dynamic-property-punt");
checkNoThrow("[\\q{a|b}]", "v", "dynamic-v-string-punt");
// `v` class strings need class context and a `{` opener.
checkThrows("\\" + "q", "v", "dynamic-v-q-bare");
checkThrows("\\" + "qx", "v", "dynamic-v-q-text");
checkThrows("\\q" + "{a}", "v", "dynamic-v-q-outside");
checkThrows("[\\" + "q]", "v", "dynamic-v-q-bare-class");
// Atom-only escapes are illegal inside classes.
checkThrows("[\\k" + "<a>]", "u", "dynamic-class-k");
checkThrows("(a)[\\" + "k<a>]", "u", "dynamic-class-k-group");
checkThrows("[\\" + "1]", "u", "dynamic-class-decimal");
checkThrows("(a)[\\" + "1]", "u", "dynamic-class-decimal-group");
// Unicode classes may be empty, so `]` always closes; `[]`/`[^]` continue.
checkThrows("[]" + "]", "u", "dynamic-empty-close-u");
checkThrows("[]" + "]", "v", "dynamic-empty-close-v");
checkNoThrow("[" + "]", "u", "dynamic-empty-continue");
checkNoThrow("[^" + "]", "u", "dynamic-empty-negated-continue");
if (RegExp("[" + "]", "u").test("x") !== false) throw "dynamic-empty-never";
if (RegExp("[^" + "]", "u").test("]") !== true) throw "dynamic-empty-negated-all";
checkThrows("\\" + "-", "v", "dynamic-v-dash-atom");
// Character-class escapes are never range endpoints.
checkThrows("[\\d" + "-x]", "u", "dynamic-range-start-escape");
checkThrows("[a-" + "\\d]", "u", "dynamic-range-end-escape");
checkThrows("[\\p{Lu" + "}-z]", "u", "dynamic-range-start-property");
checkThrows("[a-\\p{" + "Lu}]", "u", "dynamic-range-end-property");
checkNoThrow("[\\d" + "-]", "u", "dynamic-range-trailing-dash");
checkNoThrow("[" + "-\\d]", "u", "dynamic-range-leading-dash");
// `v` dash/amp operators: require operands and reject mixed operator families.
// Valid homogeneous operators have independent runtime capability controls.
checkThrows("[a" + "-]", "v", "dynamic-v-lone-dash-trailing");
checkThrows("[" + "-a]", "v", "dynamic-v-lone-dash-leading");
checkThrows("[a-" + "-]", "v", "dynamic-v-dash2-no-right");
checkThrows("[--" + "b]", "v", "dynamic-v-dash2-no-left");
checkThrows("[a&" + "&]", "v", "dynamic-v-and2-no-right");
checkThrows("[&&" + "b]", "v", "dynamic-v-and2-no-left");
checkThrows("[a&&" + "&b]", "v", "dynamic-v-and3");
checkThrows("[a-" + "b-c]", "v", "dynamic-v-range-chain");
checkThrows("[a-" + "b&&c]", "v", "dynamic-v-range-and-mix");
checkThrows("[a&&" + "b-c]", "v", "dynamic-v-and-range-mix");
checkThrows("[&a" + "&&b]", "v", "dynamic-v-amp-coexist");
checkNoThrow("[a" + "&b]", "v", "dynamic-v-amp1");
// Group balance is unconditional.
checkThrows("(" + "a", "u", "dynamic-unclosed-group");
checkThrows("a" + ")", "u", "dynamic-bare-paren");
checkThrows("[" + "a", "u", "dynamic-unclosed-class");
checkNoThrow("(a" + ")", "u", "dynamic-group");
checkNoThrow("[(" + "]", "u", "dynamic-class-paren");
// Brace shapes: lone `}` and partial `{` throw; valid quantifiers compile.
checkNoThrow("a{" + "1}", "u", "dynamic-quant-basic");
checkNoThrow("a{2" + ",3}", "u", "dynamic-quant-range");
checkNoThrow("[a]{" + "1}", "u", "dynamic-quant-class");
checkThrows("{" + "1}", "u", "dynamic-quant-no-atom");
checkThrows("a" + "}", "u", "dynamic-lone-brace");
checkThrows(String.fromCharCode(0x7d), "u", "dynamic-lone-brace-only");
checkThrows("[{]" + "}]", "u", "dynamic-class-then-lone");
checkThrows("a{1}" + "}", "u", "dynamic-quant-then-lone");
checkThrows("{" + "1", "u", "dynamic-quant-partial");
checkThrows("a{1" + "b", "u", "dynamic-quant-bad-follower");
checkThrows("\\{" + "1}", "u", "dynamic-quant-escaped-brace");
checkNoThrow("\\\\{" + "1}", "u", "dynamic-quant-backslash-atom");
// Clean dynamic patterns continue into the runtime parser and match.
var cont = RegExp("(a)" + "\\1", "u");
if (cont.exec("aa")[0] !== "aa") throw "dynamic-continue-backref";
var contq = RegExp("a{2" + ",3}", "u");
if (contq.exec("aaaa")[0] !== "aaa") throw "dynamic-continue-quant";

true;
