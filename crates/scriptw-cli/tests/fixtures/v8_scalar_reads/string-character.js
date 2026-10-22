var value = 'a\ud83d\udca9bc';
var sum = 0;
for (var i = 0; i < 2000; i++) sum = (sum + value.charAt(1).charCodeAt(0) + value.at(-2).charCodeAt(0)) | 0;
print(sum);
