use super::succeeds;

#[test]
fn native_catalog_overlays_preserve_shadowing_alias_writes_and_branch_joins() {
    succeeds(
        r#"
      var token = 'catalog_' + Date.now();
      function invoke(source) { return eval(source); }
      for (var i = 0; i < 4; i++) {
        var branch = '(function(flag) {' +
          ' var Math = { abs: function(n) { return n; } };' +
          ' var alias = Math;' +
          ' if (flag) alias.abs = function(n) { return n + 10; };' +
          ' return Math.abs(3); })(' + (i % 2) + ') /* ' + token + i + ' */';
        if (invoke(branch) !== (i % 2 ? 13 : 3)) throw new Error('branch alias');
        var shadow = '(function() { function Object() { return ' + (i + 20) +
          '; } return Object(); })() /* ' + token + i + ' */';
        if (invoke(shadow) !== i + 20) throw new Error('builtin shadow');
      }
      1;
    "#,
    );
}

#[test]
fn native_catalog_changes_do_not_escape_into_a_later_unseen_source() {
    succeeds(
        r#"
      var token = 'catalog_fresh_' + Date.now();
      function invoke(source) { return eval(source); }
      for (var i = 0; i < 3; i++) {
        var changed = '(function() { var Reflect = { get: function() { return ' +
          (i + 30) + '; } }; var alias = Reflect; alias.get = function() { return ' +
          (i + 40) + '; }; return Reflect.get(); })() /* ' + token + i + ' */';
        if (invoke(changed) !== i + 40) throw new Error('local alias');
        var pristine = 'Reflect.get({ value: ' + (i + 50) +
          ' }, "value") /* fresh ' + token + i + ' */';
        if (invoke(pristine) !== i + 50) throw new Error('catalog escaped');
      }
      1;
    "#,
    );
}
