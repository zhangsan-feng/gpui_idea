// Compare working set and private committed memory at the same startup stages.
// These counters are observational; do not trim working sets to hide allocations.
pub(crate) fn log_memory(stage: &'static str) {
    if !tracing::enabled!(tracing::Level::INFO) {
        return;
    }
    #[cfg(windows)]
    {
        use windows::Win32::System::{
            ProcessStatus::{
                GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX,
            },
            Threading::GetCurrentProcess,
        };

        let mut counters = PROCESS_MEMORY_COUNTERS_EX::default();
        let size = std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
        counters.cb = size;
        // The EX layout begins with PROCESS_MEMORY_COUNTERS; the API uses cb
        // to fill the extended PrivateUsage field for the current process.
        let result = unsafe {
            GetProcessMemoryInfo(
                GetCurrentProcess(),
                (&mut counters as *mut PROCESS_MEMORY_COUNTERS_EX)
                    .cast::<PROCESS_MEMORY_COUNTERS>(),
                size,
            )
        };
        match result {
            Ok(()) => tracing::info!(
                stage,
                working_set_bytes = counters.WorkingSetSize,
                private_bytes = counters.PrivateUsage,
                "process memory sample"
            ),
            Err(error) => tracing::debug!(stage, %error, "process memory sample unavailable"),
        }
    }
    #[cfg(not(windows))]
    let _ = stage;
}
