var array, box;
var result = JSON.parse('{"kick":0,"item":[1],"box":{"a":2}}', function (key, value) {
  print('call:' + (key === '' ? '<root>' : key) + ':' + (value === undefined ? 'undefined' : typeof value));
  if (key === 'kick') {
    array = [1, , 3];
    var prototype = Object.create(Array.prototype);
    Object.defineProperty(prototype, '1', {get:function () { print('inherited-one'); return 7; }});
    Object.setPrototypeOf(array, prototype);
    box = {get a() { print('get-a'); return 2; }, b:3};
    this.item = array;
    this.box = box;
  }
  if (this === array && key === '0') { array[2] = 30; array[3] = 99; }
  if (this === box && key === 'a') { delete box.b; box.c = 4; }
  return value;
});
print(result.item.length + ':' + result.item[1] + ':' + result.item[2] + ':' + result.item[3] + ':' + ('b' in result.box) + ':' + result.box.c);
