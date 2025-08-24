//! Tests for the sys module

use kronos::sys::*;

#[test]
fn test_handle_null() {
    let handle: VkDevice = Handle::NULL;
    assert!(handle.is_null());
    assert_eq!(handle.as_raw(), 0);
}

#[test]
fn test_handle_creation() {
    let handle: VkBuffer = Handle::from_raw(42);
    assert!(!handle.is_null());
    assert_eq!(handle.as_raw(), 42);
}

#[test]
fn test_handle_equality() {
    let h1: VkPipeline = Handle::from_raw(123);
    let h2: VkPipeline = Handle::from_raw(123);
    let h3: VkPipeline = Handle::from_raw(456);
    
    assert_eq!(h1, h2);
    assert_ne!(h1, h3);
}

#[test]
fn test_handle_copy() {
    let h1: VkQueue = Handle::from_raw(789);
    let h2 = h1; // Copy
    assert_eq!(h1, h2);
    assert_eq!(h1.as_raw(), h2.as_raw());
}

#[test]
fn test_constants() {
    assert_eq!(VK_TRUE, 1);
    assert_eq!(VK_FALSE, 0);
    assert_eq!(VK_WHOLE_SIZE, u64::MAX);
    assert_eq!(VK_QUEUE_FAMILY_IGNORED, u32::MAX);
}

#[test]
fn test_size_constants() {
    assert_eq!(VK_MAX_PHYSICAL_DEVICE_NAME_SIZE, 256);
    assert_eq!(VK_UUID_SIZE, 16);
    assert_eq!(VK_MAX_MEMORY_HEAPS, 16);
    assert_eq!(VK_MAX_MEMORY_TYPES, 32);
}

#[test]
fn test_handle_debug() {
    let handle: VkDevice = Handle::from_raw(999);
    let debug_str = format!("{:?}", handle);
    assert!(debug_str.contains("raw: 999"));
    assert!(debug_str.contains("Handle"));
}

#[test]
fn test_handle_hash() {
    use std::collections::HashMap;
    
    let handle1: VkBuffer = Handle::from_raw(100);
    let handle2: VkBuffer = Handle::from_raw(200);
    
    let mut map = HashMap::new();
    map.insert(handle1, "buffer1");
    map.insert(handle2, "buffer2");
    
    assert_eq!(map.get(&handle1), Some(&"buffer1"));
    assert_eq!(map.get(&handle2), Some(&"buffer2"));
}