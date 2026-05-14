let source = new Int8Array(10);
let cloned = new Int8Array(source);

if (cloned.length !== 10) throw "cloned length";
if (cloned === source) throw "same instance";
if (Object.getPrototypeOf(cloned) !== Int8Array.prototype) throw "prototype";

123;
