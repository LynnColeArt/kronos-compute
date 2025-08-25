//! Test thread safety of Kronos implementation

use kronos_compute::*;
use std::thread;
use std::sync::Arc;

fn main() {
    println!("Testing thread safety of Kronos structures...");
    
    // Test 1: Share handles between threads
    let instance = Arc::new(VkInstance::from_raw(1));
    let device = Arc::new(VkDevice::from_raw(2));
    let buffer = Arc::new(VkBuffer::from_raw(3));
    
    let mut threads = vec![];
    
    // Test instance handle
    {
        let instance = instance.clone();
        let thread = thread::spawn(move || {
            println!("Thread {:?} accessing instance handle: {}", 
                     thread::current().id(), instance.as_raw());
        });
        threads.push(thread);
    }
    
    // Test device handle
    {
        let device = device.clone();
        let thread = thread::spawn(move || {
            println!("Thread {:?} accessing device handle: {}", 
                     thread::current().id(), device.as_raw());
        });
        threads.push(thread);
    }
    
    // Test buffer handle
    {
        let buffer = buffer.clone();
        let thread = thread::spawn(move || {
            println!("Thread {:?} accessing buffer handle: {}", 
                     thread::current().id(), buffer.as_raw());
        });
        threads.push(thread);
    }
    
    for thread in threads {
        thread.join().unwrap();
    }
    
    // Test 2: Create structures from multiple threads
    let mut threads = vec![];
    
    for i in 0..4 {
        let thread = thread::spawn(move || {
            let handle = VkBuffer::from_raw(100 + i);
            println!("Thread {:?} created buffer: {}", 
                     thread::current().id(), handle.as_raw());
        });
        threads.push(thread);
    }
    
    for thread in threads {
        thread.join().unwrap();
    }
    
    println!("\n✓ All thread safety tests passed!");
}