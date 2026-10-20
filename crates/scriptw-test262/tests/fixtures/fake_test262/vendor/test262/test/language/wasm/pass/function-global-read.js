/*---
flags: [raw]
---*/

function readGlobal() {
  return x;
}

globalThis.x = 3;
readGlobal();
