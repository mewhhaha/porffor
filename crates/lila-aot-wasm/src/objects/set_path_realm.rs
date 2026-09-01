use crate::emit::ObjectMutationErrorRealmSource;

pub(super) enum SetPathRealmEnvironmentArgument {
    TrustedCurrentEnvironment,
    MainRealmFallback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ObjectMutationErrorRealm {
    TrustedCurrentEnvironment,
    MainRealmFallback,
}

pub(super) const fn set_path_realm_environment_argument(
    source: ObjectMutationErrorRealmSource,
) -> SetPathRealmEnvironmentArgument {
    match source {
        ObjectMutationErrorRealmSource::GlobalFallback => {
            SetPathRealmEnvironmentArgument::MainRealmFallback
        }
        ObjectMutationErrorRealmSource::StandardBuiltinEnvironment
        | ObjectMutationErrorRealmSource::SetPathHelperArgument => {
            SetPathRealmEnvironmentArgument::TrustedCurrentEnvironment
        }
    }
}

pub(super) const fn object_mutation_error_realm(
    source: ObjectMutationErrorRealmSource,
) -> ObjectMutationErrorRealm {
    match source {
        ObjectMutationErrorRealmSource::GlobalFallback => {
            ObjectMutationErrorRealm::MainRealmFallback
        }
        ObjectMutationErrorRealmSource::StandardBuiltinEnvironment
        | ObjectMutationErrorRealmSource::SetPathHelperArgument => {
            ObjectMutationErrorRealm::TrustedCurrentEnvironment
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime_helpers::RuntimeHelperId;

    #[test]
    fn object_mutation_realm_projection_excludes_ordinary_lexical_environments() {
        let trusted_helpers = RuntimeHelperId::ALL
            .iter()
            .copied()
            .filter(|helper| {
                ObjectMutationErrorRealmSource::for_runtime_helper(*helper)
                    == ObjectMutationErrorRealmSource::SetPathHelperArgument
            })
            .collect::<Vec<_>>();
        assert_eq!(
            trusted_helpers,
            vec![
                RuntimeHelperId::ObjectWrite,
                RuntimeHelperId::OrdinarySetDataOnReceiver,
                RuntimeHelperId::OrdinarySetDataOnReceiverWithFallback,
                RuntimeHelperId::OrdinarySet,
                RuntimeHelperId::OrdinarySetWithoutReceiverFallback,
            ]
        );

        for source in [
            ObjectMutationErrorRealmSource::StandardBuiltinEnvironment,
            ObjectMutationErrorRealmSource::SetPathHelperArgument,
        ] {
            match set_path_realm_environment_argument(source) {
                SetPathRealmEnvironmentArgument::TrustedCurrentEnvironment => {}
                SetPathRealmEnvironmentArgument::MainRealmFallback => {
                    panic!("trusted mutation source lost its set-path Realm argument")
                }
            }
            assert_eq!(
                object_mutation_error_realm(source),
                ObjectMutationErrorRealm::TrustedCurrentEnvironment
            );
        }

        match set_path_realm_environment_argument(ObjectMutationErrorRealmSource::GlobalFallback) {
            SetPathRealmEnvironmentArgument::MainRealmFallback => {}
            SetPathRealmEnvironmentArgument::TrustedCurrentEnvironment => {
                panic!("global mutation fallback exposed a set-path Realm argument")
            }
        }
        assert_eq!(
            object_mutation_error_realm(ObjectMutationErrorRealmSource::GlobalFallback),
            ObjectMutationErrorRealm::MainRealmFallback
        );
    }
}
