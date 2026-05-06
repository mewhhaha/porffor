let marker = new Error("marker");

try {
  "abcdef".substr({
    valueOf() {
      throw marker;
    },
  }, 1);
  false;
} catch (e) {
  e === marker;
}
