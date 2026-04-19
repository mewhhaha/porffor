/*---
flags: [raw]
---*/

let e = AggregateError([1, undefined, 3], "x");
e.errors[1];
