//! Share the existing checked OnceLock components, never substitute data.
use super::*;
use crate::IntlDataImageError;
use std::num::NonZeroUsize;
use std::sync::Mutex;

type Task = fn() -> Result<(), IntlDataImageError>;

pub(super) fn prewarm(requested: NonZeroUsize) -> Result<(), EmbeddedIntlProviderSetupError> {
    let jobs = requested
        .get()
        .min(std::thread::available_parallelism().map_or(1, NonZeroUsize::get))
        .min(12);
    if jobs <= 1 {
        return Ok(());
    }
    if std::env::var_os("LILA_WASM_TRACE").is_some() {
        eprintln!("lila wasm trace: minimal-Intl admission jobs: {jobs}");
    }
    // Start the two long dependency roots first. Other initializers borrow
    // those same checked OnceLocks, retaining their existing dependency DAG.
    let tasks: Vec<Task> = vec![
        || embedded_locale_data_image().map(drop),
        || crate::list_image::embedded_list_data_image_ref().map(|_| ()),
        || embedded_collator_data_image().map(drop),
        || crate::segmenter_image::embedded_segmenter_data_image_ref().map(|_| ()),
        || embedded_display_names_data_image().map(drop),
        || crate::relative_time_image::embedded_relative_time_data_image_ref().map(|_| ()),
        || crate::duration_image::embedded_duration_data_image_ref().map(|_| ()),
        || embedded_date_time_data_image().map(drop),
        || crate::time_zone_names_image::embedded_time_zone_names_data_image_ref().map(|_| ()),
        || embedded_native_locale_information_data_image().map(drop),
        || crate::number_image::embedded_number_profiles_data_image_ref().map(|_| ()),
        || crate::named_time_zone_image::embedded_named_time_zone_data_image_ref().map(|_| ()),
    ];
    run(tasks, jobs).map_err(EmbeddedIntlProviderSetupError::DataImage)
}

fn run(tasks: Vec<Task>, jobs: usize) -> Result<(), IntlDataImageError> {
    let work = Mutex::new(tasks);
    std::thread::scope(|scope| {
        let mut threads = Vec::with_capacity(jobs);
        let mut failure = None;
        for index in 0..jobs {
            let work = &work;
            match std::thread::Builder::new()
                .name(format!("lila-minimal-intl-admission-{index}"))
                .stack_size(64 * 1024 * 1024)
                .spawn_scoped(scope, move || initialize_pending(work))
            {
                Ok(thread) => threads.push(thread),
                Err(_) => {
                    // Preserve complete initialization under transient thread
                    // limits, with no more workers than the requested bound.
                    if let Err(error) = initialize_pending(work) {
                        failure = Some(error);
                    }
                    break;
                }
            }
        }
        for thread in threads {
            match thread.join() {
                Ok(Ok(())) => {}
                Ok(Err(error)) => {
                    failure.get_or_insert(error);
                }
                Err(_) => {
                    failure.get_or_insert_with(|| {
                        IntlDataImageError::consumer("Minimal admission worker panicked")
                    });
                }
            }
        }
        match failure {
            Some(error) => Err(error),
            None => Ok(()),
        }
    })
}

fn initialize_pending(work: &Mutex<Vec<Task>>) -> Result<(), IntlDataImageError> {
    loop {
        let task = work
            .lock()
            .map_err(|_| IntlDataImageError::consumer("Minimal admission queue poisoned"))?
            .pop();
        let Some(task) = task else {
            return Ok(());
        };
        task()?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn every_component_runs_and_any_component_failure_rejects() {
        static CALLS: AtomicUsize = AtomicUsize::new(0);
        fn counted() -> Result<(), IntlDataImageError> {
            CALLS.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }
        fn invalid() -> Result<(), IntlDataImageError> {
            Err(IntlDataImageError::consumer("invalid component control"))
        }
        run(vec![counted; 12], 4).unwrap();
        assert_eq!(CALLS.load(Ordering::Relaxed), 12);
        assert!(run(vec![counted, invalid, counted], 2).is_err());
    }

    #[test]
    fn concurrent_minimal_admission_keeps_exact_frames_and_provider_identity() {
        prewarm(NonZeroUsize::new(4).unwrap()).unwrap();
        let selection = IntlDataSelection::new(IntlCompilationProfile::Minimal);
        let concurrent = selection
            .selected_with_jobs(NonZeroUsize::new(4).unwrap())
            .unwrap();
        let ordinary = selection.selected().unwrap();
        assert!(std::ptr::eq(concurrent, ordinary));
        assert_eq!(
            concurrent.component_sections(),
            ordinary.component_sections()
        );
        assert_eq!(concurrent.identity(), ordinary.identity());
    }
}
