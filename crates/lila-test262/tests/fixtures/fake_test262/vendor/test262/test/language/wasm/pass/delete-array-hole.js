/*---
flags: [raw]
---*/

let a = [1, 2];
delete a[0];
(a.length === 2) && !(0 in a);
