/*---
flags: [raw]
---*/

let o = { _x: 0, set x(v) { this._x = v; } };
o.x = 3;
o._x;
