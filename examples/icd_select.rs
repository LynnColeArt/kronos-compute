//! Example: Enumerate ICDs and create a context bound to a selected ICD

use std::env;

use kronos_compute::api;
use kronos_compute::implementation::icd_loader;

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  icd_select list");
    eprintln!("  icd_select index <N>");
    eprintln!("  icd_select path <LIBPATH>");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();

    if args.is_empty() {
        print_usage();
        return Ok(());
    }

    match args[0].as_str() {
        "list" => {
            let list = icd_loader::available_icds();
            println!("Found {} ICD(s):", list.len());
            for (i, icd) in list.iter().enumerate() {
                println!(
                    "[{i}] {} ({}), api=0x{:x}{}",
                    icd.library_path.display(),
                    if icd.is_software { "software" } else { "hardware" },
                    icd.api_version,
                    icd.manifest_path
                        .as_ref()
                        .map(|p| format!(" manifest={}", p.display()))
                        .unwrap_or_default()
                );
            }
        }
        "index" => {
            if args.len() < 2 {
                print_usage();
                std::process::exit(2);
            }
            let idx: usize = args[1].parse()?;
            let ctx = api::ComputeContext::builder()
                .app_name("ICD Select (index)")
                .prefer_icd_index(idx)
                .build()?;
            if let Some(info) = ctx.icd_info() {
                println!(
                    "Context bound to ICD: {} ({}), api=0x{:x}",
                    info.library_path.display(),
                    if info.is_software { "software" } else { "hardware" },
                    info.api_version
                );
            }
        }
        "path" => {
            if args.len() < 2 {
                print_usage();
                std::process::exit(2);
            }
            let path = &args[1];
            let ctx = api::ComputeContext::builder()
                .app_name("ICD Select (path)")
                .prefer_icd_path(path)
                .build()?;
            if let Some(info) = ctx.icd_info() {
                println!(
                    "Context bound to ICD: {} ({}), api=0x{:x}",
                    info.library_path.display(),
                    if info.is_software { "software" } else { "hardware" },
                    info.api_version
                );
            }
        }
        _ => {
            print_usage();
            std::process::exit(2);
        }
    }

    Ok(())
}

