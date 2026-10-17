#!/usr/bin/env python3
"""Check the private Unicode Pattern scratch compactor's ownership and bounds."""
from pathlib import Path
import re
import sys
sys.dont_write_bytecode = True
PARENT = 'crates/lila-aot-wasm/src/builtins/regexp/compiler.rs'
CHILD = 'crates/lila-aot-wasm/src/builtins/regexp/compiler/compact_source.rs'
CONTRACT = 'crates/lila-aot-wasm/src/builtins/regexp/compiler/contracts.rs'
TEST = 'crates/lila-engine/tests/aot_regexp/aot_regexp_runtime_gap.rs'


def between(text, start, end):
    a = text.index(start)
    return text[a:text.index(end, a + len(start))]


def check(root, overrides=None):
    overrides = overrides or {}
    errors = []
    def read(name):
        return overrides[name] if name in overrides else (root / name).read_text()
    try:
        p, c, contract, test = map(read, [PARENT, CHILD, CONTRACT, TEST])
        assert p.count('mod compact_source;') == 1
        assert not re.search(r'pub(?:\([^)]*\))? mod compact_source;', p)
        body = between(p, 'pub(crate) fn compile_regexp_compiler_helper(', '\n    /// Retain the strict Unicode')
        calls = ['self.emit_regexp_compile_flags(', 'self.emit_regexp_transient_text(',
                 'self.emit_regexp_compact_unicode_source(', 'self.emit_regexp_unicode_prescan(',
                 'self.emit_regexp_compile_workspace(', 'self.emit_regexp_compile_units(',
                 'self.emit_regexp_runtime_parse(']
        positions = [body.index(call.removeprefix('self')) for call in calls]
        assert positions == sorted(positions)
        assert body.count('self.emit_regexp_compact_unicode_source(') == 1
        assert len(c.splitlines()) <= 160
        assert not re.search(r'\b(?:GcLocal|StringValue|StringValueSchema|RefCast|ArraySet|struct_type|array_type|emit_regexp_compile_failure)\b', c)
        assert 'struct CompactSourceCursor {' in c
        copy = between(c, '    fn copy(', '\n}\n')
        assert 'for cursor in [self.read, self.write]' in copy
        assert 'Instruction::I32Store8(FunctionBuilder::memarg8(0))' in copy
        assert copy.count('Instruction::I64Const(1)') == 1
        compact = between(c, '    pub(super) fn emit_regexp_compact_unicode_source(', '\n    fn emit_regexp_compact_leading_zeros(')
        assert compact.index('character_mode.emit_is_unicode(function);') < compact.index('cursor.copy(compiler, function);')
        assert compact.count('cursor.copy(compiler, function);') == 3
        assert compact.count('compiler.source_length.load(function);') == 3
        assert compact.count('Instruction::I64LtU') == 2
        assert 'Instruction::I64GeU' in compact
        assert 'Instruction::BrIf(1)' in compact
        assert 'for local in [cursor.read, cursor.write]' in compact
        assert compact.count('compiler.source_length.store(function);') == 1
        assert compact.index('cursor.write.load(function);') < compact.index('compiler.source_length.store(function);')
        assert 'for _ in 0..5 {' in compact
        assert compact.index("i64::from(b'\\\\')") < compact.index("i64::from(b'u')") < compact.index("i64::from(b'{')")
        zeros = c[c.index('    fn emit_regexp_compact_leading_zeros('):]
        assert zeros.index('Instruction::I64GeU') < zeros.index('self.emit_regexp_scratch_byte(')
        assert zeros.count("i64::from(b'0')") == 2
        assert zeros.count('Instruction::I64Const(1)') == 1
        assert 'compiler.source_pointer,cursor.read,1,cursor.next,function,' in re.sub(r'\s+', '', zeros)
        assert zeros.count('Instruction::BrIf(1)') == 3
        assert zeros.count('self.emit_regexp_scratch_increment(cursor.read, 1, function);') == 1
        assert 'cursor.write' not in zeros
        assert 'compiler.source_pointer,' in zeros
        assert 'Instruction::I64Const(u32::MAX as i64)' in contract
        for name in ['long_unicode_escape_workspace', 'compact_unicode_escape_domains', 'compact_unicode_escape_errors']:
            assert test.count(f'../fixtures/regexp_runtime_gap/{name}.js') == 1
        long = read('crates/lila-engine/tests/fixtures/regexp_runtime_gap/long_unicode_escape_workspace.js')
        assert "'0'.repeat(Math.pow(2, 24))" in long
        assert "['u', 'v']" in long
        assert 'actual.source === pair[0]' in long and 'reference.source === pair[1]' in long
        assert "['[' + escaped + ']', '[' + minimal + ']']" in long
        domains = read('crates/lila-engine/tests/fixtures/regexp_runtime_gap/compact_unicode_escape_domains.js')
        assert 'legacy.test(slash + \'u{\' + zeros + \'a}\')' in domains
        assert "new RegExp(paritySource, 'u')" in domains
        assert "new RegExp(stringsSource, 'v')" in domains
        assert 'named.groups.a' in domains
        invalid = read('crates/lila-engine/tests/fixtures/regexp_runtime_gap/compact_unicode_escape_errors.js')
        assert 'error instanceof SyntaxError' in invalid
        assert "'110000}'" in invalid and "'G}'" in invalid
    except (AssertionError, ValueError, OSError) as error:
        errors.append(f'private Unicode Pattern scratch contract failed: {error}')
    return errors

if __name__ == '__main__':
    errors = check(Path(sys.argv[1]))
    for error in errors:
        print(error)
    raise SystemExit(bool(errors))
