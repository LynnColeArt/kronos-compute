//! Safe API: Create context by selecting an ICD (ignored by default)

use std::env;

#[test]
#[ignore]
fn create_context_with_selected_icd() {
    if env::var("KRONOS_RUN_ICD_TESTS").ok().as_deref() != Some("1") {
        eprintln!("skipping (set KRONOS_RUN_ICD_TESTS=1)\n");
        return;
    }

    let icds = kronos_compute::implementation::icd_loader::available_icds();
    if icds.is_empty() {
        eprintln!("no ICDs available");
        return;
    }

    let ctx = kronos_compute::api::ComputeContext::builder()
        .app_name("Safe API ICD Select Test")
        .prefer_icd_index(0)
        .build();

    match ctx {
        Ok(c) => {
            let info = c.icd_info();
            println!("Context created. ICD info: {:?}", info);
        }
        Err(e) => {
            eprintln!("Context creation failed: {e}");
        }
    }
}

