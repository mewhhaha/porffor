var ok = true;

function check(value) {
  ok = ok && value;
}

function checkMethod(name, expectedLength, fn) {
  var desc = Object.getOwnPropertyDescriptor(String.prototype, name);
  check(typeof fn === "function");
  check(fn.name === name);
  check(fn.length === expectedLength);
  check(desc.value === fn);
  check(desc.writable === true);
  check(desc.enumerable === false);
  check(desc.configurable === true);
}

checkMethod("anchor", 1, String.prototype.anchor);
checkMethod("big", 0, String.prototype.big);
checkMethod("blink", 0, String.prototype.blink);
checkMethod("bold", 0, String.prototype.bold);
checkMethod("fixed", 0, String.prototype.fixed);
checkMethod("fontcolor", 1, String.prototype.fontcolor);
checkMethod("fontsize", 1, String.prototype.fontsize);
checkMethod("italics", 0, String.prototype.italics);
checkMethod("link", 1, String.prototype.link);
checkMethod("small", 0, String.prototype.small);
checkMethod("strike", 0, String.prototype.strike);
checkMethod("sub", 0, String.prototype.sub);
checkMethod("sup", 0, String.prototype.sup);

check(String.prototype.anchor.call("x", "n") === '<a name="n">x</a>');
check(String.prototype.big.call("x") === "<big>x</big>");
check(String.prototype.blink.call("x") === "<blink>x</blink>");
check(String.prototype.bold.call("x") === "<b>x</b>");
check(String.prototype.fixed.call("x") === "<tt>x</tt>");
check(String.prototype.fontcolor.call("x", "red") === '<font color="red">x</font>');
check(String.prototype.fontsize.call("x", 7) === '<font size="7">x</font>');
check(String.prototype.italics.call("x") === "<i>x</i>");
check(String.prototype.link.call("x", "https://e.test/?q=\"x\"") === '<a href="https://e.test/?q=&quot;x&quot;">x</a>');
check(String.prototype.small.call("x") === "<small>x</small>");
check(String.prototype.strike.call("x") === "<strike>x</strike>");
check(String.prototype.sub.call("x") === "<sub>x</sub>");
check(String.prototype.sup.call("x") === "<sup>x</sup>");
check("_".anchor("b") === '<a name="b">_</a>');
check("<".anchor("<") === '<a name="<"><</a>');
check("_".anchor(0x2A) === '<a name="42">_</a>');
check("_".anchor("\x22") === '<a name="&quot;">_</a>');
check("x".big() === "<big>x</big>");
check("x".blink() === "<blink>x</blink>");
check("x".bold() === "<b>x</b>");
check("x".fixed() === "<tt>x</tt>");
check("x".fontcolor("red") === '<font color="red">x</font>');
check("x".fontsize(7) === '<font size="7">x</font>');
check("x".italics() === "<i>x</i>");
check('x'.link('https://e.test/?q="x"') === '<a href="https://e.test/?q=&quot;x&quot;">x</a>');
check("x".small() === "<small>x</small>");
check("x".strike() === "<strike>x</strike>");
check("x".sub() === "<sub>x</sub>");
check("x".sup() === "<sup>x</sup>");

try {
  String.prototype.bold.call(null);
  check(false);
} catch (e) {
  check(e instanceof TypeError);
}

try {
  String.prototype.bold.call(undefined);
  check(false);
} catch (e) {
  check(e instanceof TypeError);
}

var marker = {};
try {
  String.prototype.bold.call({
    toString: function() {
      throw marker;
    },
  });
  check(false);
} catch (e) {
  check(e === marker);
}

try {
  String.prototype.link.call("x", {
    toString: function() {
      throw marker;
    },
  });
  check(false);
} catch (e) {
  check(e === marker);
}

ok;
