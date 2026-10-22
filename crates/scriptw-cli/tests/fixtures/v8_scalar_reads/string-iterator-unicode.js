var value = 'a\ud83d\udca9b\ud800c\udfff';
var sum = 0;
for (var i = 0; i < 128; i++) for (var part of value) sum = (sum + part.length + part.charCodeAt(0)) | 0;
print(sum);
