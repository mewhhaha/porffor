use super::*;

#[test]
fn innermost_target_uses_the_later_control_frame() {
    let outer = ControlTarget {
        frame: 2,
        environment_depth: 1,
        label: LabelDepth::for_test(3),
    };
    let inner = ControlTarget {
        frame: 5,
        environment_depth: 3,
        label: LabelDepth::for_test(6),
    };

    assert_eq!(innermost_target(outer, inner), inner);
    assert_eq!(innermost_target(inner, outer), inner);
}

#[test]
fn environment_hops_is_the_depth_difference() {
    assert_eq!(environment_hops(0, 0), 0);
    assert_eq!(environment_hops(4, 1), 3);
}

#[test]
#[should_panic(expected = "control target environment must enclose the current environment")]
fn environment_hops_rejects_a_deeper_target() {
    environment_hops(1, 2);
}

#[test]
fn finalizer_crosses_only_branches_to_outer_frames() {
    let finalizer = ControlTarget {
        frame: 4,
        environment_depth: 0,
        label: LabelDepth::for_test(5),
    };
    let outer_branch = ControlTarget {
        frame: 1,
        environment_depth: 0,
        label: LabelDepth::for_test(2),
    };
    let inner_branch = ControlTarget {
        frame: 6,
        environment_depth: 0,
        label: LabelDepth::for_test(7),
    };

    assert!(finalizer_crosses_branch(finalizer, outer_branch));
    assert!(!finalizer_crosses_branch(finalizer, finalizer));
    assert!(!finalizer_crosses_branch(finalizer, inner_branch));
}
