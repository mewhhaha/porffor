
const input = 'before-a-after';
let calls = 0;
let result = '';
for (let round = 0; round < 512; round++) {
  result = input.replaceAll('a', 'B');
  calls += result.length;
}

let checksum = 0;
const stride = Math.max(1, Math.floor(result.length / 32));
for (let i = 0; i < result.length; i += stride) {
  checksum = Math.imul(checksum, 31) + result.charCodeAt(i) | 0;
}
print(result.length + ':' + checksum + ':' + calls + ':' + result.slice(0, 2) + ':' + result.slice(-2));
