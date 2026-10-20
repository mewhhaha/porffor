function check(actual, expected, label) {
  if (actual !== expected) throw label;
}

check(/(?s:.)/.test("\n"), true, "force dotAll on");
check(/(?-s:.)/s.test("\n"), false, "force dotAll off");
check(/(?m:^x$)/.test("a\nx\nb"), true, "force multiline on");
check(/(?-m:^x$)/m.test("a\nx\nb"), false, "force multiline off");
check(/(?s:(?-s:.).)./.test("a\nb"), true, "restore outer dotAll override");
check(/(?s:(?-s:.).)./.test("a\n\n"), false, "restore inherited dotAll state");

true;
