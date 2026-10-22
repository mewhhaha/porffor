
const input = 'a,'.repeat(2048);
const piece = 'x'.repeat(255) + 'Y';
let calls = 0;
const replacement = '[' + piece + '$&]$$';
const result = input.replace(/a/g, replacement);

let checksum = 0;
const stride = Math.max(1, Math.floor(result.length / 32));
for (let i = 0; i < result.length; i += stride) {
  checksum = Math.imul(checksum, 31) + result.charCodeAt(i) | 0;
}
print(result.length + ':' + checksum + ':' + calls + ':' + result.slice(0, 2) + ':' + result.slice(-2));
