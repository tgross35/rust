use std::fs;
use run_make_support::{rustc, rust_lib_name, rfs};
use crate::{SYSROOT, SYSROOT_LIBS_DIR};

pub fn check_diagnostics() {
    let input = "evil_sysroot_crate.rs";
    let out_file = SYSROOT_LIBS_DIR.join(rust_lib_name(input));

    println!("creating {out_file:?}");
    
    rustc().input(input).output(&out_file).run();

    assert!(fs::exists(&out_file).unwrap(), "failed to create {out_file:?}");

    rustc().input("diagnostics.rs").run();


    // rfs::remove_file(&out_file);
    
    
}


