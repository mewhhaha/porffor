var ok = true;

function check(value) {
  ok = ok && value;
}

function checkGlobal(name, fn) {
  var desc = Object.getOwnPropertyDescriptor(globalThis, name);
  check(typeof fn === "function");
  check(fn.name === name);
  check(fn.length === 1);
  check(desc.value === fn);
  check(desc.writable === true);
  check(desc.enumerable === false);
  check(desc.configurable === true);
  check(!("prototype" in fn));
  check(__lilaIsConstructor(fn) === false);
}

checkGlobal("escape", escape);
checkGlobal("unescape", unescape);

check(escape() === "undefined");
check(escape(undefined) === "undefined");
check(escape(null) === "null");
check(escape(true) === "true");
check(escape(12) === "12");
check(escape("AZaz09@*_+-./") === "AZaz09@*_+-./");
check(escape(" !#~") === "%20%21%23%7E");
check(escape("\n\r\t") === "%0A%0D%09");
check(escape("\u0100\u0101\u0102") === "%u0100%u0101%u0102");
check(escape("\ufffd\ufffe\uffff") === "%uFFFD%uFFFE%uFFFF");
check(escape("\u{10401}") === "%uD801%uDC01");

check(unescape() === "undefined");
check(unescape(undefined) === "undefined");
check(unescape(null) === "null");
check(unescape(true) === "true");
check(unescape("%20%21%23%7e") === " !#~");
check(unescape("a%2Fb%2fc") === "a/b/c");
check(unescape("%") === "%");
check(unescape("%2") === "%2");
check(unescape("%GG") === "%GG");
check(unescape("x%2G%41") === "x%2GA");
check(unescape("%0%u002A0") === "%0*0");
check(unescape("%0%uFFFE0") === "%0\ufffe0");
check(unescape("%u0100%u0101%u0102") === "\u0100\u0101\u0102");
check(unescape("%uFFFD%uFFFE%uFFFF") === "\ufffd\ufffe\uffff");
var astral = unescape("%uD801%uDC01");
check(astral === "\u{10401}");
check(astral.length === 2);
check(astral.charCodeAt(0) === 0xd801);
check(astral.charCodeAt(1) === 0xdc01);
var loneLead = unescape("%uD801");
check(loneLead === "\uD801");
check(loneLead.length === 1);
check(loneLead.charCodeAt(0) === 0xd801);
var loneTrail = unescape("%uDC01");
check(loneTrail === "\uDC01");
check(loneTrail.length === 1);
check(loneTrail.charCodeAt(0) === 0xdc01);
check(unescape("%uD801A") === "\uD801A");
check(unescape("%uD801\uDC01") === "\u{10401}");
check(unescape("\uD801%uDC01") === "\u{10401}");
check(unescape("\u{10401}") === "\u{10401}");
check(unescape("\u0100%u00G0\u{10401}") === "\u0100%u00G0\u{10401}");
check(unescape("%u") === "%u");
check(unescape("%u0") === "%u0");
check(unescape("%u00") === "%u00");
check(unescape("%u000") === "%u000");
check(unescape("%u000G") === "%u000G");
check(unescape("%U0000") === "%U0000");

var marker = {};
try {
  escape({
    toString: function() {
      throw marker;
    },
  });
  check(false);
} catch (e) {
  check(e === marker);
}

try {
  unescape({
    toString: function() {
      throw marker;
    },
  });
  check(false);
} catch (e) {
  check(e === marker);
}

ok;
