/*---
flags: [raw]
---*/

function F(x) { this.x = x; }
let G = F.bind(null, 2);
new G().x;
