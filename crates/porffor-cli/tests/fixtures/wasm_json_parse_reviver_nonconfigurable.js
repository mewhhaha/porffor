function fail(message) {
  throw message;
}

var arrayCreate = JSON.parse("[1, 2]", function(key, value) {
  if (key === "0") {
    Object.defineProperty(this, "1", { configurable: false });
  }
  if (key === "1") return 22;
  return value;
});

if (arrayCreate[0] !== 1) fail("array create index 0");
if (arrayCreate[1] !== 2) fail("array create index 1");

var arrayDelete = JSON.parse("[1, 2]", function(key, value) {
  if (key === "0") {
    Object.defineProperty(this, "1", { configurable: false });
  }
  if (key === "1") return;
  return value;
});

if (arrayDelete[0] !== 1) fail("array delete index 0");
if (arrayDelete[1] !== 2) fail("array delete index 1");

var objectCreate = JSON.parse("{\"a\": 1, \"b\": 2}", function(key, value) {
  if (key === "a") {
    Object.defineProperty(this, "b", { configurable: false });
  }
  if (key === "b") return 22;
  return value;
});

if (objectCreate.a !== 1) fail("object create a");
if (objectCreate.b !== 2) fail("object create b");

var objectDelete = JSON.parse("{\"a\": 1, \"b\": 2}", function(key, value) {
  if (key === "a") {
    Object.defineProperty(this, "b", { configurable: false });
  }
  if (key === "b") return;
  return value;
});

if (objectDelete.a !== 1) fail("object delete a");
if (objectDelete.b !== 2) fail("object delete b");

262;
