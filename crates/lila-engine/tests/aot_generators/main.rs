//! Consolidated integration tests: generators and resumable/suspended evaluation.
//!
//! One binary per area bounds the number of linked Wasmtime test executables
//! while keeping per-process memory bounded.

mod aot_class_computed_name_suspension;
mod aot_generator_array_patterns;
mod aot_generator_call_suspension;
mod aot_generator_catch_environment;
mod aot_generator_class_abrupt_cleanup;
mod aot_generator_classic_loops;
mod aot_generator_eager_values;
mod aot_generator_for_in;
mod aot_generator_for_of_continuations;
mod aot_generator_identifier_reference;
mod aot_generator_instance_prototype;
mod aot_generator_invocation_references;
mod aot_generator_linear_assignment;
mod aot_generator_loops;
mod aot_generator_object_literal;
mod aot_generator_object_patterns;
mod aot_generator_optional_calls;
mod aot_generator_optional_regions;
mod aot_generator_pattern_assignments;
mod aot_generator_pattern_initializers;
mod aot_generator_plain_identifier_assignment;
mod aot_generator_reference_operands;
mod aot_generator_staged_operands;
mod aot_generator_switch_regions;
mod aot_generator_throw_regions;
mod aot_generator_value_branches;
mod aot_generator_with;
mod aot_private_optional_call_suspension;
mod aot_resumable_catch_patterns;
mod aot_resumable_environment_anchor;
mod aot_resumable_for_in_family;
mod aot_resumable_for_of_family;
mod aot_resumable_global_assignment;
mod aot_resumable_resource_scopes;
mod aot_suspended_call_references;
mod aot_suspended_references;
