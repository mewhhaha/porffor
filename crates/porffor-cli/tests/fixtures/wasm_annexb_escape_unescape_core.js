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
  check(__porfIsConstructor(fn) === false);
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
