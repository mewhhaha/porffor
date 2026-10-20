use super::succeeds;

#[test]
fn empty_comment_eval_reuses_wasm_across_every_utf16_code_unit() {
    let body = r#"
        function run() {
            var sentinel = 17;
            for (var i = 0; i < 65536; i++) {
                var source = '/*var ' + String.fromCharCode(i) + 'xx = 1*/';
                if (eval(source) !== undefined) throw new Error('empty completion');
            }
            var lines = [10, 13, 0x2028, 0x2029];
            for (var i = 0; i < lines.length; i++) {
                sentinel = 17;
                eval('//comment' + String.fromCharCode(lines[i]) + 'sentinel=23');
                if (sentinel !== 23) throw new Error('line terminator effect');
            }
            if (eval(String.fromCharCode(39) + 'use strict' + String.fromCharCode(39)) !== 'use strict')
                throw new Error('directive completion');
            try { eval('/*' + 'unterminated'); throw 1; }
            catch (error) { if (!(error instanceof SyntaxError)) throw error; }
        }
        run();
        1;
    "#;
    for prefix in ["", "'use strict';"] {
        succeeds(&format!("{prefix}{body}"));
    }
}
