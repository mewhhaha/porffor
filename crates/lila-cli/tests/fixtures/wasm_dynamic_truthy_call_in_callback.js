function yes() {
  return true;
}

function test(callback) {
  callback();
}

test(function () {
  if (!yes()) {
    throw "call result should be truthy";
  }
});

262;
