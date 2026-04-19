/*---
flags: [raw]
---*/

function F() { this.nt = new.target; }
let G = F.bind(null);
let x = new G();
x.nt === F;
