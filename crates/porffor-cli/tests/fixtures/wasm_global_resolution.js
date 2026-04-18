globalThis.x = 1;
x;

missing = 2;

function readGlobal() {
  return x;
}

function writeGlobal() {
  y = 3;
  return y;
}

let x = 4;
globalThis.x = 5;

readGlobal();
writeGlobal();
x;
