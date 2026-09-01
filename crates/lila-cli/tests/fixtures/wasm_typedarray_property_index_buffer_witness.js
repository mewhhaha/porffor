function assertSame(actual, expected, label) {
  if (actual !== expected) throw label;
}

var buffer = new ArrayBuffer(8, { maxByteLength: 16 });
var view = new Uint16Array(buffer, 2);

buffer.resize(5);
assertSame(view.length, 1, "odd-byte length floors");
assertSame(view[1], undefined, "odd-byte read is absent");

var conversions = "";
view[1] = {
  valueOf: function() {
    conversions = conversions + "odd,";
    return 0x1111;
  }
};
assertSame(conversions, "odd,", "odd-byte write converts");
assertSame(view[1], undefined, "odd-byte write stays absent");

view[1] = {
  valueOf: function() {
    conversions = conversions + "grow,";
    buffer.resize(6);
    return 0x2233;
  }
};
assertSame(conversions, "odd,grow,", "growth write converts once");
assertSame(view.length, 2, "growth is observed after conversion");
assertSame(view[1], 0x2233, "growth write uses the current backing store");

view[1] = {
  valueOf: function() {
    conversions = conversions + "shrink,";
    buffer.resize(5);
    return 0x3344;
  }
};
assertSame(conversions, "odd,grow,shrink,", "shrink write converts once");
assertSame(view[1], undefined, "shrink makes the write absent");

var detachedBuffer = new ArrayBuffer(4);
var detached = new Uint16Array(detachedBuffer);
detached[0] = {
  valueOf: function() {
    conversions = conversions + "detach,";
    __lilaDetachArrayBuffer(detachedBuffer);
    return 0x4455;
  }
};
assertSame(conversions, "odd,grow,shrink,detach,", "detach write converts once");
assertSame(detached.length, 0, "detached length is zero");
assertSame(detached[0], undefined, "detached read is absent");

true;
