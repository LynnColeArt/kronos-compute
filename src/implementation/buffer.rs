//! REAL Kronos buffer implementation - NO ICD forwarding!

use crate::sys::*;
use crate::core::*;
use crate::ffi::*;
use std::ptr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::collections::HashMap;

// Buffer handle counter
static BUFFER_COUNTER: AtomicU64 = AtomicU64::new(1);

// Registry of active buffers
lazy_static::lazy_static! {
    static ref BUFFERS: Mutex<HashMap<u64, BufferData>> = Mutex::new(HashMap::new());
}

struct BufferData {
    device: VkDevice,
    size: VkDeviceSize,
    usage: VkBufferUsageFlags,
    sharing_mode: VkSharingMode,
    memory: Option<VkDeviceMemory>,
    memory_offset: VkDeviceSize,
}

/// Create a buffer - REAL implementation
#[no_mangle]
pub unsafe extern "C" fn vkCreateBuffer(
    device: VkDevice,
    pCreateInfo: *const VkBufferCreateInfo,
    _pAllocator: *const VkAllocationCallbacks,
    pBuffer: *mut VkBuffer,
) -> VkResult {
    log::info!("=== KRONOS vkCreateBuffer called (Pure Rust) ===");
    
    if device.is_null() || pCreateInfo.is_null() || pBuffer.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    let create_info = &*pCreateInfo;
    
    // Validate buffer size
    if create_info.size == 0 {
        return VkResult::ErrorInitializationFailed;
    }
    
    // We only support compute/transfer usage
    let allowed_usage = VkBufferUsageFlags::STORAGE_BUFFER 
        | VkBufferUsageFlags::TRANSFER_SRC 
        | VkBufferUsageFlags::TRANSFER_DST
        | VkBufferUsageFlags::UNIFORM_BUFFER;
        
    if !allowed_usage.contains(create_info.usage) {
        log::warn!("Buffer usage {:?} contains unsupported flags", create_info.usage);
    }
    
    // Create buffer handle
    let handle = BUFFER_COUNTER.fetch_add(1, Ordering::SeqCst);
    
    // Store buffer data
    let buffer_data = BufferData {
        device,
        size: create_info.size,
        usage: create_info.usage,
        sharing_mode: create_info.sharingMode,
        memory: None,
        memory_offset: 0,
    };
    
    BUFFERS.lock().unwrap().insert(handle, buffer_data);
    
    *pBuffer = VkBuffer::from_raw(handle);
    
    log::info!("Created buffer {:?} with size {}", handle, create_info.size);
    
    VkResult::Success
}

/// Destroy a buffer
#[no_mangle]
pub unsafe extern "C" fn vkDestroyBuffer(
    device: VkDevice,
    buffer: VkBuffer,
    _pAllocator: *const VkAllocationCallbacks,
) {
    if device.is_null() || buffer.is_null() {
        return;
    }
    
    let handle = buffer.as_raw();
    BUFFERS.lock().unwrap().remove(&handle);
    
    log::info!("Destroyed buffer {:?}", handle);
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
    
    let handle = buffer.as_raw();
    if let Some(buffer_data) = BUFFERS.lock().unwrap().get(&handle) {
        let requirements = &mut *pMemoryRequirements;
        
        // Simple alignment requirement
        requirements.alignment = 256; // 256 byte alignment
        requirements.size = (buffer_data.size + 255) & !255; // Round up to alignment
        
        // We support all memory types for compute
        requirements.memoryTypeBits = 0xFFFFFFFF;
    }
}

/// Bind memory to buffer
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
    
    let handle = buffer.as_raw();
    if let Some(buffer_data) = BUFFERS.lock().unwrap().get_mut(&handle) {
        buffer_data.memory = Some(memory);
        buffer_data.memory_offset = memoryOffset;
        
        log::info!("Bound memory {:?} to buffer {:?} at offset {}", 
                   memory.as_raw(), handle, memoryOffset);
        
        VkResult::Success
    } else {
        VkResult::ErrorDeviceLost
    }
}