// Independently allocated short strings are each hashed once after retention.
// Construction, insertion and checksum all belong to the measured invocation.
(function () {
  const values = [];
  for (let i = 0; i < 32768; ++i) values.push('s' + String.fromCharCode(i & 65535));
  const map = new Map();
  for (let i = 0; i < values.length; ++i) map.set(values[i], i);
  let sum = 0;
  for (const value of map.values()) sum += value;
  if (sum !== 536854528 || map.size !== values.length) throw new Error('first hash checksum');
  print(sum);
})();
