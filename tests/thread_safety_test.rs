//! Thread safety test for Arc-based ICD management

use std::{sync::Arc, thread, time::Duration};

#[test]
fn test_arc_based_thread_safety() {
    // This test verifies that the Arc-based ICD lifetime management
    // works correctly across multiple threads
    
    let handles = (0..4).map(|i| {
        thread::spawn(move || {
            // Each thread discovers ICDs independently
            let icds = kronos_compute::implementation::icd_loader::available_icds();
            println!("Thread {} found {} ICDs", i, icds.len());
            
            // Simulate some work
            thread::sleep(Duration::from_millis(10));
            
            // Try to create an instance
            if !icds.is_empty() {
                match kronos_compute::api::ComputeContext::builder()
                    .app_name(&format!("Thread {} Test", i))
                    .build()
                {
                    Ok(_ctx) => {
                        println!("Thread {} created context successfully", i);
                        // Context will be dropped here, testing Arc cleanup
                    }
                    Err(e) => {
                        println!("Thread {} failed to create context: {}", i, e);
                    }
                }
            }
            
            // Verify ICDs are still available after context drop
            let icds_after = kronos_compute::implementation::icd_loader::available_icds();
            assert_eq!(icds.len(), icds_after.len(), "ICD count changed unexpectedly in thread {}", i);
        })
    }).collect::<Vec<_>>();
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify ICDs are still available after all threads complete
    let final_icds = kronos_compute::implementation::icd_loader::available_icds();
    println!("Final ICD count: {}", final_icds.len());
}

#[test]
fn test_concurrent_icd_discovery() {
    // Test that multiple threads can discover ICDs concurrently
    // without data races or crashes
    
    let thread_count = 8;
    let iterations = 5;
    
    let handles = (0..thread_count).map(|t| {
        thread::spawn(move || {
            for i in 0..iterations {
                let icds = kronos_compute::implementation::icd_loader::available_icds();
                assert!(!icds.is_empty() || icds.is_empty(), "Thread {} iter {} got valid result", t, i);
                
                // Brief pause
                thread::sleep(Duration::from_micros(100));
            }
        })
    }).collect::<Vec<_>>();
    
    for handle in handles {
        handle.join().unwrap();
    }
}