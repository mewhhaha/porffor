var ok = true;

function check(value) {
  ok = ok && value;
}

function checkGlobal(name, fn) {
  var descriptor = Object.getOwnPropertyDescriptor(globalThis, name);
  check(typeof fn === "function");
  check(fn.name === name);
  check(fn.length === 1);
  check(descriptor.value === fn);
  check(descriptor.writable === true);
  check(descriptor.enumerable === false);
  check(descriptor.configurable === true);
  check(!("prototype" in fn));
  check(__lilaIsConstructor(fn) === false);
}

checkGlobal("encodeURI", encodeURI);
checkGlobal("encodeURIComponent", encodeURIComponent);
checkGlobal("decodeURI", decodeURI);
checkGlobal("decodeURIComponent", decodeURIComponent);

check(encodeURI() === "undefined");
check(encodeURIComponent(undefined) === "undefined");
check(decodeURI() === "undefined");
check(decodeURIComponent(undefined) === "undefined");

var componentUnescaped = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_.!~*'()";
var uriReserved = ";/?:@&=+$,#";
check(encodeURIComponent(componentUnescaped) === componentUnescaped);
check(encodeURI(componentUnescaped + uriReserved) === componentUnescaped + uriReserved);
check(encodeURIComponent(uriReserved) === "%3B%2F%3F%3A%40%26%3D%2B%24%2C%23");
check(encodeURI("a b%\n") === "a%20b%25%0A");

check(encodeURIComponent("\u00a2\u20ac\u{10348}") === "%C2%A2%E2%82%AC%F0%90%8D%88");
check(
  encodeURIComponent(String.fromCharCode(0xd800, 0xdc00)) === "%F0%90%80%80",
);
check(decodeURIComponent("%C2%A2%E2%82%AC%F0%90%8D%88") === "\u00a2\u20ac\u{10348}");
check(decodeURIComponent("%EF%BF%BE%EF%BF%BF") === "\ufffe\uffff");
check(decodeURIComponent("%00%25%7e") === "\0%~");
check(decodeURI("%3B%2f%3f%3A%40%26%3d%2B%24%2c%23") === "%3B%2f%3f%3A%40%26%3d%2B%24%2c%23");
check(decodeURIComponent("%3B%2f%3f%3A%40%26%3d%2B%24%2c%23") === uriReserved);

var rawLoneSurrogate = String.fromCharCode(0xd800);
check(decodeURI(rawLoneSurrogate) === rawLoneSurrogate);
check(decodeURIComponent(rawLoneSurrogate) === rawLoneSurrogate);

function throwsUriError(callback, expectedConstructor) {
  try {
    callback();
  } catch (error) {
    return error instanceof expectedConstructor;
  }
  return false;
}

check(throwsUriError(function() {
  encodeURI(String.fromCharCode(0xd800));
}, URIError));
check(throwsUriError(function() {
  encodeURIComponent(String.fromCharCode(0xdc00));
}, URIError));
check(throwsUriError(function() {
  encodeURI(String.fromCharCode(0xd800) + "a");
}, URIError));

var malformed = [
  "%",
  "%2",
  "%GG",
  "%80",
  "%C0%AF",
  "%E0%80%80",
  "%ED%A0%80",
  "%ED%7F%BF",
  "%F0%80%80%80",
  "%F4%90%80%80",
  "%F5%80%80%80",
  "%E2%82",
  "%E2%82A",
];
for (var i = 0; i < malformed.length; i++) {
  var encoded = malformed[i];
  check(throwsUriError(function() {
    decodeURI(encoded);
  }, URIError));
  check(throwsUriError(function() {
    decodeURIComponent(encoded);
  }, URIError));
}

var marker = {};
for (var j = 0; j < 4; j++) {
  var codec = [encodeURI, encodeURIComponent, decodeURI, decodeURIComponent][j];
  try {
    codec({
      toString: function() {
        throw marker;
      },
    });
    check(false);
  } catch (error) {
    check(error === marker);
  }
}

var other = __lilaCreateRealm().global;
try {
  other.encodeURI(String.fromCharCode(0xd800));
  check(false);
} catch (error) {
  check(error instanceof other.URIError);
  check(!(error instanceof URIError));
}

ok;
