var arr = [1, 2];
Object.defineProperty(arr, "1", {
  get: function() {
    return "6.99";
  },
  configurable: true
});
delete arr[1];
typeof arr[1];
