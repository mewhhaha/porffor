var value = 'x';
var sum = 0;
for (var i = 0; i < 4000; i++) sum = (sum + value.charAt(0).charCodeAt(0) + value.at(-1).charCodeAt(0)) | 0;
print(sum);
