/*---
flags: [raw]
---*/

function pick(x, y) {
  if (x) {
    return y;
  }
  return null;
}

pick(true, 1);
