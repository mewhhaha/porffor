// Dynamic short strings survive until the final character-only checksum.
// None of these strings is used as a property or collection key.
(function () {
  const values = [];
  for (let i = 0; i < 32768; ++i) values.push('s' + String.fromCharCode(i & 65535));
  let sum = 0;
  for (let i = 0; i < values.length; ++i) sum += values[i].charCodeAt(1);
  if (sum !== 536854528) throw new Error('retained string checksum');
  print(sum);
})();
