use std::path::Path;

use run_make_support::{cwd, filename_contains, rfs, rust_lib_name, rustc};

use crate::{SYSROOT, sysroot_libs_dir};

pub fn check_diagnostics() {
    rfs::copy_dir_all(&*SYSROOT, "working_sysroot");
    let libs_dir = sysroot_libs_dir(&Path::new("working_sysroot"));

    let mut working_cfg_if = None;
    rfs::read_dir_entries(libs_dir, |path| {
        if filename_contains(path, "cfg_if") {
            working_cfg_if = Some(path.to_owned());
        }
    });

    let working_cfg_if = working_cfg_if.expect("cfg_if not found in sysroot");
    println!("found sysroot cfg_if at `{:?}`", working_cfg_if);

    // Replace the cfg-if with our evil version
    rustc().input("evil_cfg_if.rs").sysroot("working_sysroot").output(working_cfg_if).run();

    let out_file = "hello-world-abc";

    println!("creating {out_file:?}");

    rustc().input("diagnostics.rs").sysroot("working_sysroot").run();

    // assert!(fs::exists(&out_file).unwrap(), "failed to create {out_file:?}");

    // rustc().input("diagnostics.rs").run();

    // rfs::remove_file(&out_file);
}

#[derive(Debug)]
struct Output {}

impl Output {
    fn capture() -> Self {
        Self {}
    }

    fn analyze(self) {}
}
