//! Multi-ICD enumeration and selection smoke tests

use std::env;

#[test]
#[ignore]
fn enumerate_icds() {
    // Only run when explicitly requested
    if env::var("KRONOS_RUN_ICD_TESTS").ok().as_deref() != Some("1") {
        eprintln!("skipping (set KRONOS_RUN_ICD_TESTS=1 to run)\n");
        return;
    }

    let icds = kronos_compute::implementation::icd_loader::available_icds();
    println!("found {} ICD(s)", icds.len());
    for (i, icd) in icds.iter().enumerate() {
        println!(
            "[{i}] {} ({}), api=0x{:x}",
            icd.library_path.display(),
            if icd.is_software { "software" } else { "hardware" },
            icd.api_version
        );
    }
}

#[test]
#[ignore]
fn select_first_icd_and_init() {
    if env::var("KRONOS_RUN_ICD_TESTS").ok().as_deref() != Some("1") {
        eprintln!("skipping (set KRONOS_RUN_ICD_TESTS=1 to run)\n");
        return;
    }

    let icds = kronos_compute::implementation::icd_loader::available_icds();
    if icds.is_empty() {
        eprintln!("no ICDs available on this system");
        return;
    }

    // Prefer first ICD and initialize
    kronos_compute::implementation::icd_loader::set_preferred_icd_index(0);
    let init = kronos_compute::initialize_kronos();
    assert!(init.is_ok(), "initialize_kronos failed: {init:?}");

    let sel = kronos_compute::implementation::icd_loader::selected_icd_info();
    assert!(sel.is_some(), "no selected ICD after initialization");

    let info = sel.unwrap();
    println!(
        "selected ICD: {} ({}), api=0x{:x}",
        info.library_path.display(),
        if info.is_software { "software" } else { "hardware" },
        info.api_version
    );

    // Cleanup preference for subsequent runs
    kronos_compute::implementation::icd_loader::clear_preferred_icd();
}

