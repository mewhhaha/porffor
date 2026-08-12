/*---
flags: [raw]
---*/

let o = { _x: 0, get x() { return this._x; }, set x(v) { this._x = v; } };
o.x = 4;
o.x;
