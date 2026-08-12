var called = 0;

function touch() {
  called++;
}

touch();

if (called !== 1) throw "nested function should update script global var";

262;
