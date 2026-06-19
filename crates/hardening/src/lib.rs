/// Block core dumps and `ptrace` attachment for this process.
///
/// `PR_SET_DUMPABLE = 0` makes the kernel refuse `ptrace` from an unprivileged
/// peer and suppresses core dumps; clamping `RLIMIT_CORE` to zero is a
/// belt-and-suspenders guard for the core-dump path. Safe in any process,
/// including the GPU-backed UI.
///
/// We deliberately don't `mlock` against swap: this binary is large (it links
/// the UI toolkit), the default `RLIMIT_MEMLOCK` (8 MiB) can't be raised
/// without system config, and locking future faults risks crashing the daemon.
/// Swap protection is left to system configuration (encrypted swap, or a
/// raised `LimitMEMLOCK` in a service unit).
pub fn forbid_dumps() {
    unsafe {
        libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0);
        let no_core = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        libc::setrlimit(libc::RLIMIT_CORE, &no_core);
    }
}
