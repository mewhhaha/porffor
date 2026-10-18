use super::succeeds;

#[test]
fn annex_b_block_functions_preserve_implicit_arguments_and_explicit_shadowing() {
    succeeds(
        r#"
      function simple() {
        var before = arguments;
        { function arguments() { return 7; } if (arguments() !== 7) throw 1; }
        if (arguments === before || arguments() !== 7 || before[0] !== 11) throw 2;
      }
      function named(value) {
        var before = arguments;
        { function arguments() { return 8; } }
        if (arguments === before || arguments() !== 8 || before[0] !== value) throw 3;
      }
      function rest(...values) {
        var before = arguments;
        { function arguments() { return 9; } }
        if (arguments === before || arguments() !== 9 || before[0] !== values[0]) throw 4;
      }
      function shadowed() {
        function arguments() { return 1; }
        { function arguments() { return 2; } }
        return arguments();
      }
      function defaults(value = arguments.length) {
        function arguments() { return 1; }
        { function arguments() { return 2; } }
        return [value, arguments()];
      }
      simple(11); named(12); rest(13);
      if (shadowed() !== 2) throw 5;
      var observed = defaults();
      if (observed[0] !== 0 || observed[1] !== 2) throw 6;
      1;
    "#,
    );
}
