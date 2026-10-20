finallyExit: {
  try {
    17;
  } finally {
    42;
    break finallyExit;
  }
}
