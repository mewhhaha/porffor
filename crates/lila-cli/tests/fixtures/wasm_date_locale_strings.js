const date = new Date(0);

if (date.toLocaleDateString() !== date.toDateString()) throw "date string";
if (date.toLocaleString() !== date.toString()) throw "date and time string";
if (date.toLocaleTimeString() !== date.toTimeString()) throw "time string";

for (const [name, length] of [
  ["toLocaleDateString", 0],
  ["toLocaleString", 0],
  ["toLocaleTimeString", 0],
]) {
  const method = Date.prototype[name];
  if (method.name !== name) throw `${name} name`;
  if (method.length !== length) throw `${name} length`;

  const descriptor = Object.getOwnPropertyDescriptor(Date.prototype, name);
  if (!descriptor) throw `${name} descriptor`;
  if (!descriptor.writable) throw `${name} writable`;
  if (descriptor.enumerable) throw `${name} enumerable`;
  if (!descriptor.configurable) throw `${name} configurable`;
}

let threw = false;
try {
  Date.prototype.toLocaleString.call({});
} catch (error) {
  threw = error instanceof TypeError;
}
if (!threw) throw "incompatible receiver";

262;
