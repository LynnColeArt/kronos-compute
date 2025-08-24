//! Memory allocation and management

use std::sync::Arc;
use std::collections::HashMap;
use std::alloc::{alloc, dealloc, Layout};
use std::ptr;
use crate::sys::*;
use crate::core::*;
use crate::ffi::*;
use super::MemoryAllocation;
use super::device::DEVICES;

/// Allocate device memory
#[no_mangle]
pub unsafe extern "C" fn vkAllocateMemory(
    device: VkDevice,
    pAllocateInfo: *const VkMemoryAllocateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pMemory: *mut VkDeviceMemory,
) -> VkResult {
    if device.is_null() || pAllocateInfo.is_null() || pMemory.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Forward to real driver if enabled
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(allocate_memory) = icd.allocate_memory {
            return allocate_memory(device, pAllocateInfo, pAllocator, pMemory);
        }
    }
    
    let alloc_info = &*pAllocateInfo;
    
    // Validate structure
    if alloc_info.sType != VkStructureType::MemoryAllocateInfo {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Get device
    let devices = DEVICES.lock().unwrap();
    let device_arc = match devices.get(&device.as_raw()) {
        Some(d) => d.clone(),
        None => return VkResult::ErrorDeviceLost,
    };
    drop(devices);
    
    let mut device_locked = device_arc.lock().unwrap();
    
    // Allocate memory handle
    let handle = VkDeviceMemory::from_raw(device_locked.handle.as_raw() << 32 | device_locked.memory_allocations.len() as u64);
    
    // Create allocation
    let allocation = MemoryAllocation {
        handle,
        size: alloc_info.allocationSize,
        memory_type: alloc_info.memoryTypeIndex,
        mapped_ptr: None,
    };
    
    device_locked.memory_allocations.insert(handle.as_raw(), allocation);
    
    *pMemory = handle;
    VkResult::Success
}

/// Free device memory
#[no_mangle]
pub unsafe extern "C" fn vkFreeMemory(
    device: VkDevice,
    memory: VkDeviceMemory,
    pAllocator: *const VkAllocationCallbacks,
) {
    if device.is_null() || memory.is_null() {
        return;
    }
    
    let devices = DEVICES.lock().unwrap();
    if let Some(device_arc) = devices.get(&device.as_raw()) {
        let mut device_locked = device_arc.lock().unwrap();
        
        if let Some(allocation) = device_locked.memory_allocations.remove(&memory.as_raw()) {
            // If memory was mapped, unmap it
            if let Some(ptr) = allocation.mapped_ptr {
                let layout = Layout::from_size_align_unchecked(
                    allocation.size as usize,
                    64 // Assume 64-byte alignment for GPU memory
                );
                dealloc(ptr, layout);
            }
        }
    }
}

/// Map memory for CPU access
#[no_mangle]
pub unsafe extern "C" fn vkMapMemory(
    device: VkDevice,
    memory: VkDeviceMemory,
    offset: VkDeviceSize,
    size: VkDeviceSize,
    flags: VkMemoryMapFlags,
    ppData: *mut *mut libc::c_void,
) -> VkResult {
    if device.is_null() || memory.is_null() || ppData.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    let devices = DEVICES.lock().unwrap();
    let device_arc = match devices.get(&device.as_raw()) {
        Some(d) => d.clone(),
        None => return VkResult::ErrorDeviceLost,
    };
    drop(devices);
    
    let mut device_locked = device_arc.lock().unwrap();
    
    // First check memory type and calculate size
    let (memory_type_idx, alloc_size, is_mapped) = {
        let allocation = match device_locked.memory_allocations.get(&memory.as_raw()) {
            Some(a) => a,
            None => return VkResult::ErrorMemoryMapFailed,
        };
        
        // Check if already mapped
        if allocation.mapped_ptr.is_some() {
            return VkResult::ErrorMemoryMapFailed;
        }
        
        (allocation.memory_type, allocation.size, allocation.mapped_ptr.is_some())
    };
    
    // Check memory type supports host access
    let mem_props = &device_locked.physical_device.memory_properties;
    let mem_type = &mem_props.memoryTypes[memory_type_idx as usize];
    
    if !mem_type.propertyFlags.contains(VkMemoryPropertyFlags::HOST_VISIBLE) {
        return VkResult::ErrorMemoryMapFailed;
    }
    
    // Determine map size
    let map_size = if size == VK_WHOLE_SIZE {
        alloc_size - offset
    } else {
        size
    };
    
    // Allocate host memory to simulate device memory
    let layout = match Layout::from_size_align(map_size as usize, 64) {
        Ok(layout) => layout,
        Err(_) => return VkResult::ErrorOutOfHostMemory,
    };
    
    let ptr = alloc(layout);
    if ptr.is_null() {
        return VkResult::ErrorOutOfHostMemory;
    }
    
    // Zero initialize if coherent
    if mem_type.propertyFlags.contains(VkMemoryPropertyFlags::HOST_COHERENT) {
        ptr::write_bytes(ptr, 0, map_size as usize);
    }
    
    // Store mapped pointer - get allocation again as mutable
    let allocation = device_locked.memory_allocations.get_mut(&memory.as_raw()).unwrap();
    allocation.mapped_ptr = Some(ptr);
    *ppData = ptr.add(offset as usize) as *mut libc::c_void;
    
    VkResult::Success
}

/// Unmap memory
#[no_mangle]
pub unsafe extern "C" fn vkUnmapMemory(
    device: VkDevice,
    memory: VkDeviceMemory,
) {
    if device.is_null() || memory.is_null() {
        return;
    }
    
    let devices = DEVICES.lock().unwrap();
    if let Some(device_arc) = devices.get(&device.as_raw()) {
        let mut device_locked = device_arc.lock().unwrap();
        
        if let Some(allocation) = device_locked.memory_allocations.get_mut(&memory.as_raw()) {
            if let Some(ptr) = allocation.mapped_ptr.take() {
                let layout = Layout::from_size_align_unchecked(
                    allocation.size as usize,
                    64
                );
                dealloc(ptr, layout);
            }
        }
    }
}

// Extension trait to make Result work with layout errors
trait LayoutResultExt {
    fn map_err<E, F: FnOnce(std::alloc::LayoutError) -> E>(self, f: F) -> Result<Layout, E>;
}

impl LayoutResultExt for Result<Layout, std::alloc::LayoutError> {
    fn map_err<E, F: FnOnce(std::alloc::LayoutError) -> E>(self, f: F) -> Result<Layout, E> {
        match self {
            Ok(layout) => Ok(layout),
            Err(e) => Err(f(e)),
        }
    }
}