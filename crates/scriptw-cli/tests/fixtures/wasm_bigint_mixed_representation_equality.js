let heapNegativeOne =
  -1n | 0x123456789abcdef0fedcba9876543210n;

heapNegativeOne === -1n &&
  Object.is(heapNegativeOne, -1n) &&
  [-1n].includes(heapNegativeOne);
