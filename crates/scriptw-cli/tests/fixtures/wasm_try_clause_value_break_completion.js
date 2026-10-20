catchExit: {
  try {
    throw 23;
  } catch (caught) {
    42;
    break catchExit;
  }
}
