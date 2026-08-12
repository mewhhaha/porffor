/*---
flags: [raw]
---*/

function F() {
  this.kind = typeof new.target;
  this.arrowKind = (() => typeof new.target)();
}

let x = new F();
x.kind === "function" && x.arrowKind === "function";
