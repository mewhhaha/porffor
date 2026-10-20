use super::*;

/// The original mixed suspension allocator, now shared by function and region
/// source factories. Every increment is checked before publishing a range.
pub(super) struct ResumableStateAllocator {
    current_state: u32,
    suspension_points: Vec<ResumableSuspensionPointIr>,
    resume_environment: ResumableResumeEnvironmentIr,
    enclosing_scope_depth: usize,
    enclosing_scope_resume_states: Vec<u32>,
}
impl ResumableStateAllocator {
    pub(super) fn at(current_state: u32) -> Self {
        Self {
            current_state,
            suspension_points: Vec::new(),
            resume_environment: ResumableResumeEnvironmentIr::SavedLexicalChain,
            enclosing_scope_depth: 0,
            enclosing_scope_resume_states: Vec::new(),
        }
    }
    pub(super) fn current(&self) -> u32 {
        self.current_state
    }
    pub(super) fn append_protocol_points(
        &mut self,
        end: u32,
        points: impl IntoIterator<Item = (ResumableSuspensionKindIr, u32, u32)>,
    ) {
        let points = points
            .into_iter()
            .map(
                |(kind, suspend_state, resume_state)| ResumableSuspensionPointIr {
                    kind,
                    suspend_state,
                    resume_state,
                    resume_environment: self.resume_environment,
                },
            )
            .collect();
        self.append_region(end, points, Vec::new());
    }
    pub(super) fn suspend(
        &mut self,
        kind: ResumableSuspensionKindIr,
    ) -> Result<(), AsyncGeneratorSourceError> {
        let suspend_state = self.current_state;
        self.reserve()?;
        self.suspension_points.push(ResumableSuspensionPointIr {
            kind,
            suspend_state,
            resume_state: self.current_state,
            resume_environment: self.resume_environment,
        });
        if self.enclosing_scope_depth > 0 {
            self.enclosing_scope_resume_states.push(self.current_state);
        }
        Ok(())
    }
    pub(super) fn reserve(&mut self) -> Result<(), AsyncGeneratorSourceError> {
        self.current_state = self
            .current_state
            .checked_add(1)
            .ok_or(AsyncGeneratorSourceError::StateOverflow)?;
        Ok(())
    }
    pub(super) fn with_resume_environment<T>(
        &mut self,
        environment: ResumableResumeEnvironmentIr,
        append: impl FnOnce(&mut Self) -> Result<T, AsyncGeneratorSourceError>,
    ) -> Result<T, AsyncGeneratorSourceError> {
        let previous = std::mem::replace(&mut self.resume_environment, environment);
        let result = append(self);
        self.resume_environment = previous;
        result
    }
    pub(super) fn reserve_async_disposable_finalizer(
        &mut self,
    ) -> Result<(), AsyncGeneratorSourceError> {
        self.reserve_resource_finalizer(self.current_state)
            .map(|_| ())
    }
    pub(super) fn reserve_resource_finalizer(
        &mut self,
        entry: u32,
    ) -> Result<AsyncDisposableFinalizerPlanIr, AsyncGeneratorSourceError> {
        self.finish_resource_finalizer(entry, self.current_state)
    }
    pub(super) fn finish_resource_finalizer(
        &mut self,
        entry: u32,
        end: u32,
    ) -> Result<AsyncDisposableFinalizerPlanIr, AsyncGeneratorSourceError> {
        if self.current_state != end && end.checked_add(1) != Some(self.current_state) {
            return Err(AsyncGeneratorSourceError::ForeignContinuation);
        }
        let plan = AsyncDisposableFinalizerPlanIr::after_source_suffix(entry, end)
            .ok_or(AsyncGeneratorSourceError::StateOverflow)?;
        if self.enclosing_scope_depth > 0 {
            self.enclosing_scope_resume_states.push(plan.resume_state());
        }
        self.current_state = plan.exit_state();
        Ok(plan)
    }
    pub(super) fn with_enclosing_scope<T>(
        &mut self,
        append: impl FnOnce(&mut Self) -> Result<T, AsyncGeneratorSourceError>,
    ) -> Result<T, AsyncGeneratorSourceError> {
        let previous = self.enclosing_scope_depth;
        self.enclosing_scope_depth = previous
            .checked_add(1)
            .ok_or(AsyncGeneratorSourceError::StateOverflow)?;
        let result = append(self);
        self.enclosing_scope_depth = previous;
        result
    }
    pub(super) fn into_tape(self) -> (Vec<ResumableSuspensionPointIr>, Vec<u32>) {
        (self.suspension_points, self.enclosing_scope_resume_states)
    }
    pub(super) fn append_region(
        &mut self,
        end: u32,
        points: Vec<ResumableSuspensionPointIr>,
        mut scoped_states: Vec<u32>,
    ) {
        self.current_state = end;
        if self.enclosing_scope_depth > 0 {
            scoped_states.extend(points.iter().map(|point| point.resume_state));
            scoped_states.sort_unstable();
            scoped_states.dedup();
        }
        self.enclosing_scope_resume_states.extend(scoped_states);
        self.suspension_points.extend(points);
    }
    pub(super) fn finish(
        self,
    ) -> Result<(u32, Vec<ResumableSuspensionPointIr>, Vec<u32>), AsyncGeneratorSourceError> {
        Ok((
            self.current_state
                .checked_add(1)
                .ok_or(AsyncGeneratorSourceError::StateOverflow)?,
            self.suspension_points,
            self.enclosing_scope_resume_states,
        ))
    }
}
