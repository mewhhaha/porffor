function fn(a) {
  let b = 1;
  var c = 1;
  const d = 1;

  {
    const a = 2;
    const b = 2;
    const c = 2;
    const d = 2;
    if (a !== 2) throw "inner a";
    if (b !== 2) throw "inner b";
    if (c !== 2) throw "inner c";
    if (d !== 2) throw "inner d";
  }

  if (a !== 1) throw "outer a";
  if (b !== 1) throw "outer b";
  if (c !== 1) throw "outer c";
  if (d !== 1) throw "outer d";
  return true;
}

fn(1);
