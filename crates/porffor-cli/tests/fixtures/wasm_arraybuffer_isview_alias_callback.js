var isView = ArrayBuffer.isView;

function testWithTypedArrayConstructors(callback) {
  callback(Int8Array);
}

testWithTypedArrayConstructors(function(ctor) {
  var sample = new ctor();
  if (isView(sample) !== true) throw "typed array callback";
});
