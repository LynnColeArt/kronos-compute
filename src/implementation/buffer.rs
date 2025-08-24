//! Buffer creation and management

use std::sync::Arc;
use crate::sys::*;
use crate::core::*;
use crate::ffi::*;
use super::Buffer;
use super::device::DEVICES;

/// Create a buffer
#[no_mangle]
pub unsafe extern "C" fn vkCreateBuffer(
    device: VkDevice,
    pCreateInfo: *const VkBufferCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pBuffer: *mut VkBuffer,
) -> VkResult {
    if device.is_null() || pCreateInfo.is_null() || pBuffer.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    let create_info = &*pCreateInfo;
    
    // Validate structure
    if create_info.sType != VkStructureType::BufferCreateInfo {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Validate size
    if create_info.size == 0 {
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
    
    // Generate buffer handle
    let handle = VkBuffer::from_raw(device_locked.handle.as_raw() << 32 | device_locked.buffers.len() as u64);
    
    // Create buffer
    let buffer = Buffer {
        handle,
        size: create_info.size,
        usage: create_info.usage,
        memory: None,
        offset: 0,
    };
    
    device_locked.buffers.insert(handle.as_raw(), buffer);
    
    *pBuffer = handle;
    VkResult::Success
}

/// Destroy a buffer
#[no_mangle]
pub unsafe extern "C" fn vkDestroyBuffer(
    device: VkDevice,
    buffer: VkBuffer,
    pAllocator: *const VkAllocationCallbacks,
) {
    if device.is_null() || buffer.is_null() {
        return;
    }
    
    let devices = DEVICES.lock().unwrap();
    if let Some(device_arc) = devices.get(&device.as_raw()) {
        let mut device_locked = device_arc.lock().unwrap();
        device_locked.buffers.remove(&buffer.as_raw());
    }
}

/// Get buffer memory requirements
#[no_mangle]
pub unsafe extern "C" fn vkGetBufferMemoryRequirements(
    device: VkDevice,
    buffer: VkBuffer,
    pMemoryRequirements: *mut VkMemoryRequirements,
) {
    if device.is_null() || buffer.is_null() || pMemoryRequirements.is_null() {
        return;
    }
    
    let devices = DEVICES.lock().unwrap();
    if let Some(device_arc) = devices.get(&device.as_raw()) {
        let device_locked = device_arc.lock().unwrap();
        
        if let Some(buffer_obj) = device_locked.buffers.get(&buffer.as_raw()) {
            // Calculate requirements based on usage
            let alignment = if buffer_obj.usage.contains(VkBufferUsageFlags::UNIFORM_BUFFER) {
                256 // Uniform buffers need 256-byte alignment
            } else {
                64 // Default alignment for storage buffers
            };
            
            // Align size up
            let aligned_size = (buffer_obj.size + alignment - 1) & !(alignment - 1);
            
            *pMemoryRequirements = VkMemoryRequirements {
                size: aligned_size,
                alignment,
                memoryTypeBits: 0x7, // Support first 3 memory types
            };
        }
    }
}

/// Bind buffer to memory
#[no_mangle]
pub unsafe extern "C" fn vkBindBufferMemory(
    device: VkDevice,
    buffer: VkBuffer,
    memory: VkDeviceMemory,
    memoryOffset: VkDeviceSize,
) -> VkResult {
    if device.is_null() || buffer.is_null() || memory.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    let devices = DEVICES.lock().unwrap();
    let device_arc = match devices.get(&device.as_raw()) {
        Some(d) => d.clone(),
        None => return VkResult::ErrorDeviceLost,
    };
    drop(devices);
    
    let mut device_locked = device_arc.lock().unwrap();
    
    // Verify memory allocation exists
    if !device_locked.memory_allocations.contains_key(&memory.as_raw()) {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Bind buffer to memory
    if let Some(buffer_obj) = device_locked.buffers.get_mut(&buffer.as_raw()) {
        if buffer_obj.memory.is_some() {
            return VkResult::ErrorUnknown; // Already bound
        }
        
        buffer_obj.memory = Some(memory);
        buffer_obj.offset = memoryOffset;
        
        VkResult::Success
    } else {
        VkResult::ErrorInitializationFailed
    }
}