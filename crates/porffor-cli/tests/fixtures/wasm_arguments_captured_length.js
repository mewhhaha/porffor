var captured;

function capture() {
  captured = arguments;
}

capture();
if (captured.length !== 0) throw "empty length";

capture(1, 2, 3);
if (captured.length !== 3) throw "filled length";

262;
