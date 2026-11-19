#![cfg(not(skip_unsupported_miri_tests))] // process spawning is not supported in Miri

use crate::os::unix::process::{ChildExt, CommandExt, ExitStatusExt, sig};
use crate::panic::catch_unwind;
use crate::process::{Command, ExitStatus};

// Many of the other aspects of this situation, including heap alloc concurrency
// safety etc., are tested in tests/ui/process/process-panic-after-fork.rs

#[test]
fn exitstatus_debug_display_tests() {
    // In practice this is the same on every Unix.
    // If some weird platform turns out to be different, and this test fails, use `cfg!`.
    use crate::os::unix::process::ExitStatusExt;
    use crate::process::ExitStatus;

    let t = |v, disp| {
        let status = <ExitStatus as ExitStatusExt>::from_raw(v);
        assert_eq!(disp, format!("{status}"), "from value: {v:#x}");
        assert_eq!(
            format!("ExitStatus(unix_wait_status({v}, {disp:?}))"),
            format!("{status:?}"),
            "from value: {v:#x}"
        );
    };

    t(0x0000f, "signal: 15 (SIGTERM)");
    t(0x0008b, "signal: 11 (SIGSEGV) (core dumped)");
    t(0x00000, "exit status: 0");
    t(0x0ff00, "exit status: 255");

    // On MacOS, 0x0137f is WIFCONTINUED, not WIFSTOPPED. Probably *BSD is similar.
    //   https://github.com/rust-lang/rust/pull/82749#issuecomment-790525956
    // The purpose of this test is to test our string formatting, not our understanding of the wait
    // status magic numbers. So restrict these to Linux.
    if cfg!(target_os = "linux") {
        if cfg!(any(target_arch = "mips", target_arch = "mips64")) {
            t(0x0137f, "stopped (not terminated) by signal: 19 (SIGPWR)");
        } else if cfg!(any(target_arch = "sparc", target_arch = "sparc64")) {
            t(0x0137f, "stopped (not terminated) by signal: 19 (SIGCONT)");
        } else {
            t(0x0137f, "stopped (not terminated) by signal: 19 (SIGSTOP)");
        }

        t(0x0ffff, "continued (WIFCONTINUED)");
    }

    // Testing "unrecognised wait status" is hard because the wait.h macros typically
    // assume that the value came from wait and isn't mad. With the glibc I have here
    // this works:
    if cfg!(all(target_os = "linux", target_env = "gnu")) {
        t(0x000ff, "unrecognised wait status: 255 0xff");
    }
}

#[test]
#[cfg_attr(target_os = "emscripten", ignore)]
#[cfg_attr(
    any(target_os = "tvos", target_os = "watchos", target_os = "l4re"),
    ignore = "fork is prohibited"
)]
fn test_command_fork_no_unwind() {
    let got = catch_unwind(|| {
        let mut c = Command::new("echo");
        c.arg("hi");
        unsafe {
            c.pre_exec(|| panic!("{}", "crash now!"));
        }
        let st = c.status().expect("failed to get command status");
        dbg!(st);
        st
    });
    dbg!(&got);
    let status = got.expect("panic unexpectedly propagated");
    dbg!(status);
    let signal = status.signal().expect("expected child process to die of signal");
    assert!(
        signal == libc::SIGABRT
            || signal == libc::SIGILL
            || signal == libc::SIGTRAP
            || signal == libc::SIGSEGV
    );
}

/// Test sending signals both to individual processes and to groups.
#[test]
fn test_signals() {
    // Inspect status, check for stopped and continued.
    let running_status = |pid: u32| {
        let mut stat = 0;
        let r = unsafe {
            libc::waitpid(
                pid.cast_signed(),
                &mut stat,
                libc::WNOHANG | libc::WUNTRACED | libc::WCONTINUED,
            )
        };
        assert!(r >= 0, "{r}");
        ExitStatus::from_raw(stat)
    };

    for (sig, action) in sig::ALL.iter().copied() {
        println!("checking signal {sig}");
        let mut single = Command::new("sleep").arg("1").spawn().unwrap();
        let mut grouped1 = Command::new("sleep").arg("1").process_group(0).spawn().unwrap();
        let mut grouped2 = Command::new("sleep")
            .arg("1")
            .process_group(grouped1.id().cast_signed())
            .spawn()
            .unwrap();

        match action {
            sig::Action::T | sig::Action::A => {
                // Terminating signals are simple; send the signal and make sure that's
                // the exit reason.
                single.send_signal(sig).unwrap();
                grouped1.send_process_group_signal(sig).unwrap();

                assert_eq!(single.wait().unwrap().signal(), Some(sig));
                assert_eq!(grouped1.wait().unwrap().signal(), Some(sig));
                assert_eq!(grouped2.wait().unwrap().signal(), Some(sig));
            }
            // Nothing to test for ignored signals.
            sig::Action::I => (),
            // These are also ignored as there is no terminal interaction
            sig::Action::S if sig == sig::TTIN || sig == sig::TTOU => (),
            sig::Action::S => {
                // "Stopping" signals should report a `WIFSTOPPED` value while running.
                single.send_signal(sig).unwrap();
                grouped1.send_process_group_signal(sig).unwrap();

                assert_eq!(running_status(single.id()).stopped_signal(), Some(sig));
                assert_eq!(running_status(grouped1.id()).stopped_signal(), Some(sig));
                assert_eq!(running_status(grouped2.id()).stopped_signal(), Some(sig));
            }
            sig::Action::C => {
                // `SIGCONT` should set `WIFCONTINUED`. Linux seems to only set this if
                // the process gets a stop signal before continuing, and it gets a wait
                // call while stopped.
                single.send_signal(sig::STOP).unwrap();
                grouped1.send_process_group_signal(sig::STOP).unwrap();

                assert_eq!(running_status(single.id()).stopped_signal(), Some(sig::STOP));
                assert_eq!(running_status(grouped1.id()).stopped_signal(), Some(sig::STOP));
                assert_eq!(running_status(grouped2.id()).stopped_signal(), Some(sig::STOP));

                single.send_signal(sig).unwrap();
                grouped1.send_process_group_signal(sig).unwrap();

                assert!(running_status(single.id()).continued());
                assert!(running_status(grouped1.id()).continued());
                assert!(running_status(grouped2.id()).continued());
            }
        }

        // Sending to an already-awaited child shouldn't error.
        single.send_signal(sig).unwrap();
        grouped1.send_signal(sig).unwrap();
        grouped2.send_signal(sig).unwrap();

        // Clean up any that didn't already exit.
        single.kill().unwrap();
        grouped1.kill().unwrap();
        grouped2.kill().unwrap();
        single.wait().unwrap();
        grouped1.wait().unwrap();
        grouped2.wait().unwrap();
    }
}
