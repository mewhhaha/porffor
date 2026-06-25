globalThis.globalX = 1;
globalX;

missing = 2;

function readGlobal() {
  return globalX;
}

function writeGlobal() {
  y = 3;
  return y;
}

let localX = 4;
globalThis.localX = 5;

readGlobal();
writeGlobal();
localX;
