var preserved = 37;
if (eval("23; for (var preserved; false;) {}") !== undefined || preserved !== 37)
  throw new Error('absent initializer or zero-iteration completion');
if (eval("23; for (var count = 0;;) { break; }") !== undefined || count !== 0)
  throw new Error('empty break completion');
if (eval("for (var count = 0; count < 3; count++) { count; }") !== 2 || count !== 3)
  throw new Error('body completion with head update');
if (eval("for (var count = 0; count < 3;) { ++count; }") !== 3 || count !== 3)
  throw new Error('prefix progress');
if (eval("for (var count = 0; count < 3;) { count += 1; }") !== 3 || count !== 3)
  throw new Error('compound progress');
if (eval("for (var count = 3; count > 0;) { count = count - 1; }") !== 0 || count !== 0)
  throw new Error('assignment progress');
print('ok');
true;
