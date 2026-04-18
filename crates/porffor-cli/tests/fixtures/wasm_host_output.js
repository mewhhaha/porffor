let alias = print;
globalThis.print("root");
alias("alias");
({ f: print }).f("method");
