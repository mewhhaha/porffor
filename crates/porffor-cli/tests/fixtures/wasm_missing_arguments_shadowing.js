function parameterShadow(a) {
  let b = 1;
  var c = 1;
  const d = 1;

  (function(a, b, c, d) {
    a = 2;
    b = 2;
    c = 2;
    d = 2;
    if (a !== 2) throw "inner parameter a";
    if (b !== 2) throw "inner parameter b";
    if (c !== 2) throw "inner parameter c";
    if (d !== 2) throw "inner parameter d";
  })(1, 1);

  if (a !== 1) throw "outer parameter a";
  if (b !== 1) throw "outer parameter b";
  if (c !== 1) throw "outer parameter c";
  if (d !== 1) throw "outer parameter d";
}

function catchShadow() {
  var c = 1;
  try {
    throw "caught";
  } catch (c) {
    (function(c) {
      c = 3;
      if (c !== 3) throw "inner catch";
    })();
    if (c !== "caught") throw "catch binding";
  }
  if (c !== 1) throw "outer catch";
}

parameterShadow(1);
catchShadow();
true;
