var captured;

function capture() {
  captured = arguments;
}

capture();
if (captured.length !== 0) throw "empty length";

capture(1, 2, 3);
if (captured.length !== 3) throw "filled length";

function writePastEnd(a, b) {
  arguments[2] = 12;
  return arguments.length === 2 && arguments[0] === 9 && arguments[1] === 11 && arguments[2] === 12;
}

if (!writePastEnd(9, 11)) throw "indexed write length";

262;
