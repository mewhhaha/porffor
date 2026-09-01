catchExit: {
  try {
    throw 17;
  } catch (caught) {
    break catchExit;
  }
}
