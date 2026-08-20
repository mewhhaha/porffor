/*---
flags: [raw]
---*/

function inc(x) {
  return x + 1;
}

let o = { f: inc };
o["f"](2);
