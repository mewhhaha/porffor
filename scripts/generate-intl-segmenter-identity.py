#!/usr/bin/env python3
"""Bind the finite Segmenter proposal to verified official ICU source archives.

This exports identity/profile metadata, never segmentation tables or handcoded
word labels. Native segmentation consumes the locked ICU image provider.
"""
import argparse
import hashlib
import json
import pathlib
import re
import tarfile
import tomllib

DATA = pathlib.Path("crates/lila-intl/data/segmenter-icu-2")
NATIVE = pathlib.Path("crates/lila-intl/src/segmenter")
LOCALES = ["ar", "ar-EG", "de", "el", "en", "en-US", "fi", "fr", "hi", "it", "ja", "ko", "sr", "sv", "zh", "zh-Hans", "zh-Hans-CN"]
PINS = {
    "icu_segmenter": ("2.0.1", "38e30e593cf9c3ca2f51aa312eb347cd1ba95715e91a842ec3fc9058eab2af4b"),
    "icu_segmenter_data": ("2.0.0", "5360a2fbe97f617c4f8b944356dedb36d423f7da7f13c070995cf89e59f01220"),
}
MARKERS = ["segmenter_break_grapheme_cluster_v1", "segmenter_break_word_v1", "segmenter_break_sentence_v1", "segmenter_break_word_override_v1", "segmenter_break_sentence_override_v1", "segmenter_dictionary_auto_v1", "segmenter_lstm_auto_v1"]
CORPORA = {"GraphemeBreakTest.txt": 1093, "WordBreakTest.txt": 1826, "SentenceBreakTest.txt": 512}

def digest(data):
    return hashlib.sha256(data).hexdigest()

def packed(value):
    return (json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(",", ":")) + "\n").encode()

def pretty(value):
    return (json.dumps(value, sort_keys=True, ensure_ascii=False, indent=2) + "\n").encode()

def require(condition, reason):
    if not condition:
        raise ValueError(reason)

def verified_manifest(root):
    raw = (root / DATA / "manifest.json").read_bytes()
    manifest = json.loads(raw)
    require(manifest["schema"] == "segmenter-icu-upstream-source/v1", "manifest schema")
    require(manifest["algorithm"] == {"name": "icu_segmenter", "version": "2.0.1"}, "algorithm identity")
    require(manifest["data"] == {"name": "icu_segmenter_data", "version": "2.0.0", "cldr": "47.0.0", "icu_export": "icu4x/2025-05-01/77.x", "lstm": "v0.1.0", "unicode": "16.0.0"}, "data identity")
    require(manifest["consumed_data_members"] == [f"data/{m}.rs.data" for m in MARKERS], "marker closure")
    require(manifest["required_complex_models"] == {"lstm": ["Burmese_", "Khmer_", "Lao_", "Thai_"], "dictionary": ["cjdict"]}, "model closure")
    require(manifest["required_tailored_overrides"] == {"word": ["fi", "sv"], "sentence": ["el"]}, "locale override closure")
    require(manifest["feature_proposal"]["required"] == ["compiled_data", "auto"] and manifest["feature_proposal"]["actual_manifest_edit"] is False, "explicit pending auto proposal")
    lock = tomllib.loads((root / "Cargo.lock").read_text())
    extracted = {}
    require([p["name"] for p in manifest["packages"]] == list(PINS), "exact package domain")
    for package in manifest["packages"]:
        name = package["name"]
        version, sha = PINS[name]
        archive = f"{name}-{version}.crate"
        require(package["version"] == version and package["sha256"] == sha, "package pin")
        require(package["archive"] == f"upstream/{archive}" and package["url"] == f"https://static.crates.io/crates/{name}/{archive}", "official archive provenance")
        require(any(p["name"] == name and p["version"] == version and p.get("checksum") == sha for p in lock["package"]), "resolved lock package checksum")
        source = root / DATA / package["archive"]
        require(source.stat().st_size == package["bytes"] and digest(source.read_bytes()) == sha, "archive checksum")
        members = {}
        with tarfile.open(source) as tar:
            for member in tar.getmembers():
                if not member.isfile():
                    continue
                path = pathlib.PurePosixPath(member.name)
                require(path.parts[0] == f"{name}-{version}" and ".." not in path.parts and len(path.parts) > 1, "unsafe archive member")
                relative = pathlib.PurePosixPath(*path.parts[1:]).as_posix()
                require(relative not in members, "duplicate archive member")
                data = tar.extractfile(member).read()
                members[relative] = {"bytes": len(data), "sha256": digest(data)}
                extracted[(name, relative)] = data
        require(members == package["source_rows"], "complete archive source rows")
        require(json.loads(extracted[(name, ".cargo_vcs_info.json")]) == package["vcs"], "upstream VCS identity")
        require((root / DATA / "licenses" / f"{name}-LICENSE").read_bytes() == extracted[(name, "LICENSE")], "original package license")
    require([c["path"] for c in manifest["corpora"]] == [f"corpora/{name}" for name in CORPORA], "exact corpus domain")
    for corpus in manifest["corpora"]:
        name = pathlib.PurePosixPath(corpus["path"]).name
        data = (root / DATA / corpus["path"]).read_bytes()
        require(data == extracted[("icu_segmenter", f"tests/testdata/{name}")], "corpus archive bytes")
        require(digest(data) == corpus["sha256"] and len(data) == corpus["bytes"], "corpus checksum")
        require(corpus["unicode_version"] == "16.0.0" and corpus["vectors"] == CORPORA[name], "corpus identity")
        require(f"# {name[:-4]}-16.0.0.txt" in data.decode().splitlines()[:4], "actual Unicode corpus version header")
        vectors = [line.split("#", 1)[0].strip() for line in data.decode().splitlines()]
        require(sum(bool(line) for line in vectors) == CORPORA[name], "all normative vector count")
        for line in filter(None, vectors):
            tokens = line.split()
            require(len(tokens) % 2 == 1 and tokens[0] == tokens[-1] == "÷", "normative boundary syntax")
            for i, token in enumerate(tokens):
                require(token in ["÷", "×"] if i % 2 == 0 else 0 <= int(token, 16) <= 0x10ffff, "normative code point")
    require(manifest["normative_vectors"] == sum(CORPORA.values()) == 3431, "total normative corpus")
    for relative in manifest["consumed_data_members"]:
        require(("icu_segmenter_data", relative) in extracted, "consumed data member missing")
    sources = tomllib.loads(extracted[("icu_segmenter_data", "Cargo.toml.orig")].decode())["package"]["metadata"]["sources"]
    require(sources == {"cldr": {"tagged": "47.0.0"}, "icuexport": {"tagged": "icu4x/2025-05-01/77.x"}, "segmenter_lstm": {"tagged": "v0.1.0"}}, "actual baked-data upstream source metadata")
    features = tomllib.loads(extracted[("icu_segmenter", "Cargo.toml.orig")].decode())["features"]
    require(features["auto"] == ["lstm"] and features["lstm"] == ["dep:core_maths"], "actual auto feature dependency")
    require(any(p["name"] == "core_maths" and p["version"] == "0.1.1" for p in lock["package"]), "locked auto dependency foundation")
    return raw, manifest

KERNEL_ROOT_PACKAGES = {
    "icu_segmenter": "2.0.1", "icu_segmenter_data": "2.0.0", "icu_locale": "2.0.0",
    "icu_provider": "2.0.0", "serde": "1.0.228", "serde_json": "1.0.149", "sha2": "0.10.9",
    "icu_provider_blob": "2.0.0", "icu_provider_adapters": "2.0.0",
}

def locked_packages(root):
    """The actual selected dependency graph, including auto's core_maths/libm."""
    lock = tomllib.loads((root / "Cargo.lock").read_text())["package"]
    by_name = {}
    for row in lock:
        by_name.setdefault(row["name"], []).append(row)
    def select(name, version=None):
        candidates = [r for r in by_name.get(name, []) if version is None or r["version"] == version]
        require(len(candidates) == 1, f"ambiguous or missing selected package: {name} {version}")
        row = candidates[0]
        require(row.get("source") == "registry+https://github.com/rust-lang/crates.io-index" and len(row.get("checksum", "")) == 64, f"unbound registry dependency: {name}")
        return row
    queue = [select(name, version) for name, version in KERNEL_ROOT_PACKAGES.items()]
    selected = {}
    while queue:
        row = queue.pop(); key = (row["name"], row["version"])
        if key in selected:
            continue
        selected[key] = {k: row[k] for k in ("name", "version", "source", "checksum")}
        for dependency in row.get("dependencies", []):
            words = dependency.split()
            queue.append(select(words[0], words[1] if len(words) > 1 else None))
    require(("core_maths", "0.1.1") in selected and ("libm", "0.2.16") in selected, "complete auto math dependency closure")
    return [selected[key] for key in sorted(selected)]

def kernel_sources(root):
    sources = [pathlib.Path(name) for name in (
        "Cargo.toml", "Cargo.lock", "crates/lila-intl/Cargo.toml",
        "crates/lila-intl/src/lib.rs", "crates/lila-intl/src/provider.rs", "crates/lila-intl/src/provider/conformance.rs", "crates/lila-intl/src/selection.rs",
        "crates/lila-intl/src/service_selection.rs",
        "crates/lila-intl/src/selection/manifest.rs",
        "crates/lila-intl/src/selection/export.rs",
        "crates/lila-intl/src/protocol.rs", "crates/lila-intl/src/supported_values.rs",
        "crates/lila-intl/src/identifiers.rs", "crates/lila-intl/src/number_format/options.rs",
        "crates/lila-intl/src/number_protocol.rs", "scripts/generate-intl-segmenter-identity.py",
        "crates/lila-intl/src/segmenter.rs", "crates/lila-intl/src/segmenter_protocol.rs",
        "crates/lila-engine/src/intl_segmenter_host.rs", "crates/lila-engine/src/wasm_gc_intl_host.rs",
        "crates/lila-engine/src/intl_data_images.rs", "crates/lila-intl/build.rs",
        "crates/lila-intl/src/image.rs", "crates/lila-intl/src/locale_image.rs",
        "crates/lila-intl/src/image_build/locale.rs", "crates/lila-intl/src/segmenter_image.rs",
        "crates/lila-intl/src/image_build/segmenter.rs",
    )]
    sources += [pathlib.Path(name) for name in ['crates/lila-intl/src/image_build/keyword.rs', 'crates/lila-intl/src/provider/keyword_aliases.rs', 'crates/lila-intl/src/provider/keyword_aliases/generated.rs', 'crates/lila-intl/data/cldr-47-bcp47/manifest.json', 'crates/lila-intl/src/provider/language_domain.rs']]
    sources += [pathlib.Path(name) for name in ['crates/lila-aot-wasm/src/emit.rs', 'crates/lila-aot-wasm/src/emit/module_assembly.rs']]
    sources += [p.relative_to(root) for p in sorted((root / "crates/lila-intl/src/segmenter_image").rglob("*.rs"))
        if p.name != "tests.rs" and "tests" not in p.relative_to(root / "crates/lila-intl/src/segmenter_image").parts]
    for owner in (NATIVE, pathlib.Path("crates/lila-intl/src/segmenter_protocol"), pathlib.Path("crates/lila-intl/src/number_protocol")):
        sources += [p.relative_to(root) for p in sorted((root / owner).rglob("*.rs"))
            if p.name not in ("kernel_identity.rs", "tests.rs") and "tests" not in p.relative_to(root / owner).parts]
    sources += [NATIVE / "profile.json", NATIVE / "profile_identity.rs"]
    sources += [p.relative_to(root) for p in sorted((root / DATA).rglob("*")) if p.is_file() and p.name != "kernel-source-manifest.json"]
    return sorted(set(sources))

def outputs(root):
    manifest_bytes, manifest = verified_manifest(root)
    profile = {"schema_version": 1, "algorithm_version": "2.0.1", "data_version": "2.0.0", "unicode_version": "16.0.0", "source_manifest_sha256": digest(manifest_bytes), "default_locale": "en-US", "locales": LOCALES}
    profile_bytes = packed(profile)
    constants = (f'// Generated by scripts/generate-intl-segmenter-identity.py.\n'
                 f'pub const SEGMENTER_DATA_SHA256: &str =\n    "{digest(profile_bytes)}";\n'
                 f'pub(super) const SEGMENTER_SOURCE_MANIFEST_SHA256: &str =\n    "{digest(manifest_bytes)}";\n').encode()
    generated = {NATIVE / "profile.json": profile_bytes, NATIVE / "profile_identity.rs": constants}
    cargo_manifest = tomllib.loads((root / "crates/lila-intl/Cargo.toml").read_text())
    cargo = cargo_manifest["dependencies"]["icu_segmenter"]
    build = cargo_manifest["build-dependencies"]["icu_segmenter"]
    data = cargo_manifest["build-dependencies"]["icu_segmenter_data"]
    require(cargo["version"] == "=2.0.1" and cargo["default-features"] is False and set(cargo["features"]) == {"serde", "auto"}, "actual image serde/auto manifest admission")
    require(build["version"] == "=2.0.1" and build["default-features"] is False and set(build["features"]) == {"datagen", "auto"}, "actual datagen/auto build admission")
    require(data["version"] == "=2.0.0" and data["default-features"] is False, "actual exact build data admission")
    sources = kernel_sources(root)
    packages = locked_packages(root)
    rows = []
    for path in sorted(set(sources)):
        data = generated.get(path)
        if data is None:
            data = (root / path).read_bytes()
        rows.append({"path": path.as_posix(), "bytes": len(data), "sha256": digest(data)})
    recipe = {"schema": "segmenter-source-kernel/v1", "wire_version": 1, "operations_reserved": [33, 34, 35], "source_rows": rows, "source_manifest_sha256": digest(manifest_bytes), "profile_sha256": digest(profile_bytes), "host_call_abi": int(re.fullmatch(r'.*?pub const INTL_HOST_CALL_ABI_VERSION: u16 = (\d+);.*', (root / 'crates/lila-intl/src/lib.rs').read_text(), re.S).group(1)), "packages": packages, "actual_feature_admission": {"runtime": ["serde", "auto"], "build": ["datagen", "auto"]}, "runtime_claim": "source-complete image/shared/native/host adapter; actual compile/native/product verification pending"}
    recipe_bytes = packed(recipe)
    generated[DATA / "kernel-source-manifest.json"] = pretty({**recipe, "kernel_sha256": digest(recipe_bytes)})
    generated[NATIVE / "kernel_identity.rs"] = (f'// Generated by scripts/generate-intl-segmenter-identity.py.\npub const SEGMENTER_KERNEL_SHA256: &str =\n    "{digest(recipe_bytes)}";\n').encode()
    return generated

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=pathlib.Path, default=pathlib.Path(__file__).resolve().parents[1])
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    root = args.root.resolve()
    generated = outputs(root)
    for path, data in generated.items():
        target = root / path
        if args.check:
            require(target.read_bytes() == data, f"non-reproducible {path}")
        else:
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
    print(f"Segmenter {'checked' if args.check else 'generated'} {len(generated)} metadata outputs; 17 service-local locales; all 3431 Unicode16 vectors bound; no native execution")

if __name__ == "__main__":
    main()
