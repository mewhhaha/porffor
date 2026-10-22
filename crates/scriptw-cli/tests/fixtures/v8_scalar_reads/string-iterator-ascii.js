var value = 'abcdefgh';
var sum = 0;
for (var i = 0; i < 128; i++) for (var part of value) sum = (sum + part.charCodeAt(0)) | 0;
print(sum);
