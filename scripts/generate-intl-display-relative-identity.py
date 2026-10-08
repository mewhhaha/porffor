#!/usr/bin/env python3
"""Bind the consumed DisplayNames and RelativeTimeFormat kernels and pinned data."""
import argparse
import hashlib
import json
import re
from pathlib import Path
import tomllib

ROOT = Path(__file__).resolve().parents[1]
SHARED = {
    'Cargo.toml', 'crates/lila-intl/Cargo.toml',
    'crates/lila-intl/src/lib.rs', 'crates/lila-intl/src/provider.rs', 'crates/lila-intl/src/provider/conformance.rs', 'crates/lila-intl/src/selection.rs',
    'crates/lila-intl/src/service_selection.rs',
    'crates/lila-intl/src/selection/manifest.rs',
    'crates/lila-intl/src/selection/export.rs',
    'crates/lila-intl/src/protocol.rs', 'crates/lila-intl/src/identifiers.rs',
    'crates/lila-intl/src/supported_values.rs',
    'crates/lila-intl/src/number_protocol.rs',
    'crates/lila-intl/src/image.rs', 'crates/lila-intl/src/locale_image.rs',
    'crates/lila-intl/src/image_build/locale.rs', 'crates/lila-intl/build.rs',
    'crates/lila-engine/src/intl_data_images.rs',
    'crates/lila-engine/src/wasm_gc_intl_host.rs',
    'scripts/generate-intl-display-relative-identity.py',
}
SHARED.update(['crates/lila-intl/src/image_build/keyword.rs', 'crates/lila-intl/src/provider/keyword_aliases.rs', 'crates/lila-intl/src/provider/keyword_aliases/generated.rs', 'crates/lila-intl/data/cldr-47-bcp47/manifest.json', 'crates/lila-intl/src/provider/language_domain.rs'])
SHARED.update(['crates/lila-aot-wasm/src/emit.rs', 'crates/lila-aot-wasm/src/emit/module_assembly.rs'])
PACKAGES = {
    'icu_locale': '2.0.0', 'icu_locale_core': '2.0.1',
    'icu_locale_data': '2.0.0', 'icu_provider': '2.0.0',
    'icu_provider_blob': '2.0.0', 'icu_provider_adapters': '2.0.0',
    'serde': '1.0.228', 'serde_json': '1.0.149', 'sha2': '0.10.9',
}
RELATIVE_PACKAGES = {**PACKAGES, 'ryu-js': '1.0.2'}


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def production_sources(repository, owner, suffix='*.rs'):
    return {
        p.relative_to(repository).as_posix()
        for p in (repository / owner).rglob(suffix)
        if 'tests' not in p.relative_to(repository / owner).parts
        and p.name not in ('tests.rs', 'kernel_identity.rs')
        and not p.name.endswith('_tests.rs')
    }


def check_manifest(repository, relative, *, root_relative=False):
    document = json.loads((repository / relative).read_text())
    rows = document['files']
    if isinstance(rows, dict):
        rows = [dict(row, path=path) for path, row in rows.items()]
    paths = {relative}
    for row in rows:
        path = Path(row['path'])
        if path.is_absolute() or '..' in path.parts:
            raise ValueError('invalid captured source path')
        if not root_relative:
            path = Path(relative).parent / path
        if path.as_posix() in paths:
            raise ValueError('duplicate captured source path')
        raw = (repository / path).read_bytes()
        if sha(raw) != row['sha256'] or len(raw) != row['bytes']:
            raise ValueError(f'captured input changed: {path}')
        paths.add(path.as_posix())
    return paths


def packages(repository, required):
    lock = tomllib.loads((repository / 'Cargo.lock').read_text())
    result = []
    for name, version in sorted(required.items()):
        rows = [p for p in lock['package'] if p['name'] == name and p['version'] == version]
        if len(rows) != 1 or rows[0].get('source') != 'registry+https://github.com/rust-lang/crates.io-index':
            raise ValueError(f'wrong locked dependency: {name}')
        row = rows[0]
        if len(row.get('checksum', '')) != 64:
            raise ValueError(f'missing locked checksum: {name}')
        result.append({k: row[k] for k in ('name', 'version', 'source', 'checksum')})
    return result


def generate(repository):
    repository = repository.resolve()
    common = SHARED | production_sources(repository, 'crates/lila-intl/src/provider/language_domain')
    common |= production_sources(repository, 'crates/lila-intl/src/provider/keyword_aliases')
    common |= {'crates/lila-intl/src/provider/language_domain.rs',
               'crates/lila-intl/src/provider/keyword_aliases.rs'}
    common |= production_sources(repository, 'crates/lila-intl/src/number_protocol')
    common |= production_sources(repository, 'scripts/intl_cldr_profile', '*.py')
    common |= {'scripts/intl_cldr_profile.py', 'scripts/intl_ldml_schema.py',
               'scripts/intl_positional_numbering.py', 'scripts/intl_calendar_eras.py'}
    dn = common | production_sources(repository, 'crates/lila-intl/src/display_names')
    dn |= production_sources(repository, 'crates/lila-intl/src/display_names_image')
    dn |= {'crates/lila-intl/src/display_names.rs', 'crates/lila-intl/src/display_names_protocol.rs',
           'crates/lila-intl/src/display_names_image.rs',
           'crates/lila-intl/src/display_names/profile.json',
           'crates/lila-intl/src/display_names/provenance.json',
           'scripts/generate-intl-displaynames-profile.py',
           'crates/lila-engine/src/intl_display_names_host.rs'}
    dn |= check_manifest(repository, 'crates/lila-intl/data/datetime-cldr-47/manifest.json')
    dn |= check_manifest(repository, 'crates/lila-intl/data/cldr-47-bcp47/manifest.json')
    dn |= check_manifest(repository, 'crates/lila-intl/data/calendar-eras-cldr-48/manifest.json')
    dn |= check_manifest(repository, 'crates/lila-intl/data/numbering-tols-cldr-48/source-manifest.json')
    rtf = common | production_sources(repository, 'crates/lila-intl/src/relative_time_format')
    rtf |= production_sources(repository, 'crates/lila-intl/src/relative_time_protocol')
    rtf |= production_sources(repository, 'crates/lila-intl/src/relative_time_image')
    rtf |= production_sources(repository, 'crates/lila-intl/src/number_image')
    rtf |= production_sources(repository, 'crates/lila-intl/src/number_format')
    rtf |= production_sources(repository, 'crates/lila-intl/src/plural_rules')
    rtf |= {'crates/lila-intl/src/relative_time_format.rs', 'crates/lila-intl/src/relative_time_protocol.rs',
            'crates/lila-intl/src/number_operation.rs', 'crates/lila-intl/src/plural_protocol.rs',
            'crates/lila-intl/src/relative_time_image.rs', 'crates/lila-intl/src/number_image.rs',
            'crates/lila-intl/data/number-cldr-47/payload-manifest.json',
            'crates/lila-intl/src/number_format/profiles/fingerprint.rs',
            'crates/lila-intl/src/relative_time_format/generated/profile.json',
            'crates/lila-intl/data/number-cldr-47/profiles.bin',
            'crates/lila-intl/data/relative-time-cldr-47/report.json',
            'scripts/generate-intl-relative-time-profile.py',
            'crates/lila-engine/src/intl_relative_time_host.rs'}
    rtf |= production_sources(repository, 'crates/lila-intl/src/plural_protocol')
    rtf |= check_manifest(repository, 'crates/lila-intl/data/relative-time-cldr-47/source-inputs.json', root_relative=True)
    outputs = {}
    for service, paths, constant, data_dir, source_dir, required in (
        ('DisplayNames', dn, 'DISPLAY_NAMES_KERNEL_SHA256', 'displaynames-cldr-47', 'display_names', PACKAGES),
        ('RelativeTimeFormat', rtf, 'RELATIVE_TIME_KERNEL_SHA256', 'relative-time-cldr-47', 'relative_time_format', RELATIVE_PACKAGES),
    ):
        locked = packages(repository, required)
        records = []
        for path in sorted(paths):
            raw = (repository / path).read_bytes()
            records.append({'path': path, 'bytes': len(raw), 'sha256': sha(raw)})
        recipe = {'schema_version': 1, 'service': service, 'host_call_abi': int(re.fullmatch(r'.*?pub const INTL_HOST_CALL_ABI_VERSION: u16 = (\d+);.*', (repository / 'crates/lila-intl/src/lib.rs').read_text(), re.S).group(1)),
                  'wire_version': 1, 'files': records, 'packages': locked,
                  'scope': 'actual immutable image admission, retained selected data owners, checked native algorithm, primitive codecs, host consumer and captured CLDR inputs; tests and self identities excluded; runtime acceptance pending'}
        digest = hashlib.sha256(json.dumps(recipe, sort_keys=True, separators=(',', ':')).encode()).digest()
        rust = '// Generated by scripts/generate-intl-display-relative-identity.py; do not edit.\n'
        rust += f'pub(crate) const {constant}: [u8; 32] = [\n'
        for start in range(0, 32, 16):
            rust += '    ' + ', '.join(f'0x{b:02x}' for b in digest[start:start + 16]) + ',\n'
        rust += '];\n'
        outputs[f'crates/lila-intl/src/{source_dir}/kernel_identity.rs'] = rust
        outputs[f'crates/lila-intl/data/{data_dir}/kernel-manifest.json'] = json.dumps(
            dict(recipe, provider_data_sha256=digest.hex()), sort_keys=True, indent=2) + '\n'
    return outputs


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repository', type=Path, default=ROOT)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    outputs = generate(args.repository)
    for relative, content in outputs.items():
        path = args.repository / relative
        if args.check:
            if path.read_text() != content:
                raise ValueError(f'stale component identity: {relative}')
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content)
    print(json.dumps({json.loads(content)['service']: {'sha256': json.loads(content)['provider_data_sha256'],
                                                     'source_files': len(json.loads(content)['files'])}
                      for path, content in outputs.items() if path.endswith('.json')}, sort_keys=True))


if __name__ == '__main__':
    main()
