#!/usr/bin/env python3
"""Check portable/native source-compilation authority at its actual owners."""
from pathlib import Path
import re
import sys

A = "crates/lila-aot-wasm/src/"
E = "crates/lila-engine/src/"
FILES = tuple(A + name for name in (
    "runtime_artifact.rs", "runtime_artifact/source_compilation.rs",
    "runtime_artifact/cache.rs", "runtime_artifact/build_package.rs",
    "runtime_artifact/validated_profile.rs", "emit/module_assembly.rs",
    "gc_types/host.rs", "prepared_script.rs", "functions/native_direct_eval.rs",
    "builtins/function/dynamic_constructor.rs",
)) + tuple(E + name for name in (
    "lib.rs", "embedded_runtime.rs", "embedded_runtime/build.rs",
    "wasm_runtime_link.rs", "wasm_native_source_compiler.rs", "graph_observation.rs",
))


def compact(source):
    return re.sub(r"\s+", "", re.sub(r"//[^\n]*", "", source))


def check(root, overrides=None):
    source = {name: overrides[name] if overrides and name in overrides
              else (Path(root) / name).read_text() for name in FILES}
    c = {name: compact(text) for name, text in source.items()}
    failures = []

    def require(condition, message):
        if not condition:
            failures.append(message)

    profile = c[A + "runtime_artifact/source_compilation.rs"]
    require("#[default]Portable,NativeHost," in profile and
            "[0]=>Some(Self::Portable),[1]=>Some(Self::NativeHost),_=>None" in profile,
            "closed profile defaults Portable and decodes exactly one 0/1 byte")
    require("ifself.profile.is_some(){returnErr(" in profile and
            "self.imports[slot]" in profile and
            'import.module!="lila_host"' in profile and
            "TypeRef::Func(_)|TypeRef::FuncExact(_)" in profile and
            "RuntimeSourceCompilationProfile::Portable=>[false;3]" in profile and
            "RuntimeSourceCompilationProfile::NativeHost=>[true;3]" in profile and
            "ifself.imports!=expected" in profile,
            "duplicate/unknown frames and wrong, partial, duplicate or foreign compiler imports reject")
    for name in ("compile_dynamic_function", "compile_global_script", "compile_direct_eval"):
        require('"' + name + '"' in profile, "all three compiler roles have closed admission")
    runtime = c[A + "runtime_artifact.rs"]
    require("Self::Program(runtime)=>runtime.source_compilation_profile()" in runtime and
            "Self::Standalone=>RuntimeSourceCompilationProfile::Portable" in runtime and
            "cache::RuntimeIdentity::admit(selection,inputs.source_compilation_profile())?" in runtime and
            "emit_runtime_artifact(selection,inputs.source_compilation_profile())" in runtime,
            "P inherits its actual R and residency resolves the complete selected profile")
    cache = c[A + "runtime_artifact/cache.rs"]
    require("sections.insert(RUNTIME_SOURCE_COMPILATION_PROFILE_CUSTOM_SECTION," in cache and
            "source_compilation_profile.wire()" in cache and
            "source_admission.import(&import).ok()?" in cache and
            "source_admission.section(section.data()).ok()?" in cache and
            "source_admission.finish().ok()?!=identity.source_compilation_profile" in cache and
            "RuntimeArtifact::new(Arc::from(payload),layout,version,identity.source_compilation_profile,)" in cache,
            "identity and full raw admission bind physical frame plus exact import capability")
    package = c[A + "runtime_artifact/build_package.rs"]
    require("source_compilation_profile:RuntimeSourceCompilationProfile::Portable" in package and
            "build_runtime_artifact_with_source_compilation_profile(selection,compiler_source,RuntimeSourceCompilationProfile::Portable,)" in package and
            "RuntimeIdentity::admit(selection,source_compilation_profile)?" in package and
            "identity.digest()" in package and "cache::decode_payload(payload,identity,version)?" in package,
            "ordinary build packages default Portable and retain profile-separated admission")
    assembly = c[A + "emit/module_assembly.rs"]
    require("uses_heap,kind.source_compilation_profile()," in assembly and
            "data:Cow::Owned(vec![kind.source_compilation_profile().wire()])" in assembly,
            "actual module role owns both import planning and physical profile frame")
    require("|GcHostImport::CompileDirectEval=>{source_compilation_profile==crate::RuntimeSourceCompilationProfile::NativeHost}" in c[A + "gc_types/host.rs"],
            "compiler hooks are selected solely by NativeHost authority")
    for name in ("prepared_script.rs", "functions/native_direct_eval.rs", "builtins/function/dynamic_constructor.rs"):
        require("gc_host_imports().source_compilation_profile()" in c[A + name] and
                "RuntimeSourceCompilationProfile::Portable" in c[A + name] and
                "emit_reject_dynamic_source(" in c[A + name],
                "Portable miss remains an execution-time typed rejection: " + name)
    engine = c[E + "lib.rs"]
    require("self.emit_wasm_with_cache_on_current_thread(unit,cache.as_deref(),lila_aot_wasm::RuntimeSourceCompilationProfile::Portable,)" in engine and
            "self.emit_wasm_with_cache_on_current_thread(&unit,cache.as_deref(),lila_aot_wasm::RuntimeSourceCompilationProfile::NativeHost,)?" in engine,
            "public export is Portable while real execution explicitly opts into NativeHost")
    require("RuntimeSourceCompilationProfile::NativeHost" in c[E + "graph_observation.rs"] and
            "build_runtime_artifact_with_source_compilation_profile(&selection,&source,lila_aot_wasm::RuntimeSourceCompilationProfile::NativeHost,)" in c[E + "embedded_runtime/build.rs"],
            "graph observation and native image production explicitly retain native compilation")
    embedded = c[E + "embedded_runtime.rs"]
    require("ifsource_compilation_profile==RuntimeSourceCompilationProfile::Portable{returninputs;}" in embedded and
            "with_source_compilation_profile(source_compilation_profile)" in embedded,
            "an embedded native bundle never becomes a Portable candidate")
    link = c[E + "wasm_runtime_link.rs"]
    require(link.count("RuntimeSourceCompilationProfile::from_module(program).ok()?==lila_aot_wasm::RuntimeSourceCompilationProfile::Portable") == 2 and
            "RuntimeSourceCompilationProfile::from_module(program.bytes)" in link and
            "profile!=runtime.source_compilation_profile()" in link,
            "both compiler-owned standalone decoders and actual linked P/R check physical authority")
    require("runtime.source_compilation_profile()!=lila_aot_wasm::RuntimeSourceCompilationProfile::NativeHost" in c[E + "wasm_native_source_compiler.rs"],
            "a native callback context cannot be minted from Portable R")
    return failures


if __name__ == "__main__":
    failures = check(Path(sys.argv[1]) if len(sys.argv) > 1 else Path.cwd())
    for failure in failures:
        print("runtime source profile: " + failure, file=sys.stderr)
    raise SystemExit(bool(failures))
