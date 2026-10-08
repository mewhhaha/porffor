var injectedArray, injectedObject;
function source(context) {
  return Object.prototype.hasOwnProperty.call(context, 'source') ? context.source : 'none';
}
var result = JSON.parse('{"start":0,"array":[1,2],"object":{"old":3}}', function (key, value, context) {
  print((key === '' ? '<root>' : key) + ':' + source(context));
  if (key === 'start') {
    injectedArray = [{inserted:4}, 5, 6];
    injectedObject = {fresh:{deep:7}, last:8};
    this.array = injectedArray;
    this.object = injectedObject;
  }
  return value;
});
print((result.array === injectedArray) + ':' + (result.object === injectedObject) + ':' + result.array.length + ':' + result.object.fresh.deep);
