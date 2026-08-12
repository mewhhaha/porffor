var host = {
  byteLengthOf: function (buffer) {
    return buffer.byteLength;
  },
};

function callHost(buffer) {
  return host.byteLengthOf(buffer);
}

let buffer = new ArrayBuffer(6);
callHost(buffer);
