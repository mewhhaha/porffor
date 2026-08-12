/*---
flags: [raw]
---*/

try { Function.prototype.toString.call({}); } catch (e) { e instanceof TypeError; }
