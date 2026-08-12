function expectJSON(text) {
  JSON.parse(text);
}

function expectSyntaxError(text) {
  var threw = false;

  try {
    JSON.parse(text);
  } catch (error) {
    threw = error instanceof SyntaxError;
  }

  if (!threw) throw "expected SyntaxError";
}

expectJSON("[]");
expectJSON("[1]");
expectJSON('{"a":1}');
expectJSON("[1,{}]");
expectSyntaxError("[1,]");
expectSyntaxError("[");
expectSyntaxError("[1");
expectSyntaxError("[1,{");
expectSyntaxError("[1,{]");
expectSyntaxError('{"a":1,}');
expectSyntaxError('{"a"}');
expectSyntaxError('{"a":}');
expectSyntaxError("this");
expectSyntaxError('"Unterminated string literal');
expectSyntaxError('["Unclosed array"');
expectSyntaxError('{unquoted_key: "keys must be quoted"}');
expectSyntaxError('["extra comma",]');
expectSyntaxError('["double extra comma",,]');
expectSyntaxError('[   , "<-- missing value"]');
expectSyntaxError('["Comma after the close"],');
expectSyntaxError('["Extra close"]]');
expectSyntaxError('{"Extra comma": true,}');
expectSyntaxError('{"Extra value after close": true} "misplaced quoted value"');
expectSyntaxError('{"Illegal expression": 1 + 2}');
expectSyntaxError('{"Illegal invocation": alert()}');
expectSyntaxError('{"Numbers cannot be hex": 0x14}');
expectSyntaxError('["Illegal backslash escape: \\x15"]');
expectSyntaxError('[\\naked]');
expectSyntaxError('["Illegal backslash escape: \\017"]');
expectSyntaxError('{"Missing colon" null}');
expectSyntaxError('{"Double colon":: null}');
expectSyntaxError('{"Comma instead of colon", null}');
expectSyntaxError('["Colon instead of comma": false]');
expectSyntaxError('["Bad value", truth]');
expectSyntaxError("['single quote']");
expectSyntaxError('["\ttab\tcharacter\tin\tstring\t"]');
expectSyntaxError('["tab\\   character\\   in\\  string\\  "]');
expectSyntaxError('["line\rbreak"]');
expectSyntaxError('["line\nbreak"]');
expectSyntaxError('["line\r\nbreak"]');
expectSyntaxError('["line\\\rbreak"]');
expectSyntaxError('["line\\\nbreak"]');
expectSyntaxError('["line\\\r\nbreak"]');
expectSyntaxError("[0e]");
expectSyntaxError("[0e+]");
expectSyntaxError("[0e+-1]");
expectSyntaxError('{"Comma instead of closing brace": true,');
expectSyntaxError('["mismatch"}');
expectSyntaxError("0{");

262;
