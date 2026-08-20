/*---
flags: [raw]
---*/

function writeGlobal() {
  y = 4;
}

writeGlobal();
globalThis.y;
