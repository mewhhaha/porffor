let marker = new Error("marker");

try {
  "abcdef".substr(1, {
    valueOf() {
      throw marker;
    },
  });
  false;
} catch (e) {
  e === marker;
}
