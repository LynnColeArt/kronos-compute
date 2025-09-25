//! Descriptor set management implementation

use crate::sys::*;
use crate::core::*;
use crate::ffi::*;
use crate::implementation::icd_loader;

/// Create descriptor set layout
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. pCreateInfo points to a valid VkDescriptorSetLayoutCreateInfo structure
// 3. pAllocator is either null or points to valid allocation callbacks
// 4. pSetLayout points to valid memory for writing the layout handle
// 5. All binding descriptions in pCreateInfo are valid
// 6. Descriptor types and shader stages are appropriate for compute
#[no_mangle]
pub unsafe extern "C" fn vkCreateDescriptorSetLayout(
    device: VkDevice,
    pCreateInfo: *const VkDescriptorSetLayoutCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pSetLayout: *mut VkDescriptorSetLayout,
) -> VkResult {
    if device.is_null() || pCreateInfo.is_null() || pSetLayout.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    if let Some(icd) = icd_loader::icd_for_device(device) {
        if let Some(f) = icd.create_descriptor_set_layout { return f(device, pCreateInfo, pAllocator, pSetLayout); }
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(create_descriptor_set_layout) = icd.create_descriptor_set_layout { return create_descriptor_set_layout(device, pCreateInfo, pAllocator, pSetLayout); }
    }
    VkResult::ErrorInitializationFailed
}

/// Destroy descriptor set layout
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. descriptorSetLayout is a valid VkDescriptorSetLayout, or VK_NULL_HANDLE
// 3. pAllocator matches the allocator used in vkCreateDescriptorSetLayout
// 4. No descriptor sets using this layout are currently allocated
// 5. No pipelines reference this layout
#[no_mangle]
pub unsafe extern "C" fn vkDestroyDescriptorSetLayout(
    device: VkDevice,
    descriptorSetLayout: VkDescriptorSetLayout,
    pAllocator: *const VkAllocationCallbacks,
) {
    if device.is_null() || descriptorSetLayout.is_null() {
        return;
    }
    
    if let Some(icd) = icd_loader::icd_for_device(device) {
        if let Some(f) = icd.destroy_descriptor_set_layout { f(device, descriptorSetLayout, pAllocator); }
        return;
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(destroy_descriptor_set_layout) = icd.destroy_descriptor_set_layout { destroy_descriptor_set_layout(device, descriptorSetLayout, pAllocator); }
    }
}

/// Create descriptor pool
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. pCreateInfo points to a valid VkDescriptorPoolCreateInfo structure
// 3. pAllocator is either null or points to valid allocation callbacks
// 4. pDescriptorPool points to valid memory for writing the pool handle
// 5. Pool sizes and max sets are reasonable values
// 6. Descriptor types match what will be allocated from this pool
#[no_mangle]
pub unsafe extern "C" fn vkCreateDescriptorPool(
    device: VkDevice,
    pCreateInfo: *const VkDescriptorPoolCreateInfo,
    pAllocator: *const VkAllocationCallbacks,
    pDescriptorPool: *mut VkDescriptorPool,
) -> VkResult {
    if device.is_null() || pCreateInfo.is_null() || pDescriptorPool.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    if let Some(icd) = icd_loader::icd_for_device(device) {
        if let Some(f) = icd.create_descriptor_pool { return f(device, pCreateInfo, pAllocator, pDescriptorPool); }
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(create_descriptor_pool) = icd.create_descriptor_pool { return create_descriptor_pool(device, pCreateInfo, pAllocator, pDescriptorPool); }
    }
    VkResult::ErrorInitializationFailed
}

/// Destroy descriptor pool
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. descriptorPool is a valid VkDescriptorPool, or VK_NULL_HANDLE
// 3. pAllocator matches the allocator used in vkCreateDescriptorPool
// 4. All descriptor sets allocated from this pool have been freed or will be freed
// 5. No command buffers are using descriptor sets from this pool
#[no_mangle]
pub unsafe extern "C" fn vkDestroyDescriptorPool(
    device: VkDevice,
    descriptorPool: VkDescriptorPool,
    pAllocator: *const VkAllocationCallbacks,
) {
    if device.is_null() || descriptorPool.is_null() {
        return;
    }
    
    if let Some(icd) = icd_loader::icd_for_device(device) {
        if let Some(f) = icd.destroy_descriptor_pool { f(device, descriptorPool, pAllocator); }
        return;
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(destroy_descriptor_pool) = icd.destroy_descriptor_pool { destroy_descriptor_pool(device, descriptorPool, pAllocator); }
    }
}

/// Reset descriptor pool
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. descriptorPool is a valid VkDescriptorPool
// 3. flags is a valid VkDescriptorPoolResetFlags value
// 4. All descriptor sets allocated from this pool become invalid after reset
// 5. No command buffers are currently using descriptor sets from this pool
#[no_mangle]
pub unsafe extern "C" fn vkResetDescriptorPool(
    device: VkDevice,
    descriptorPool: VkDescriptorPool,
    flags: VkDescriptorPoolResetFlags,
) -> VkResult {
    if device.is_null() || descriptorPool.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    if let Some(icd) = icd_loader::icd_for_device(device) {
        if let Some(f) = icd.reset_descriptor_pool { return f(device, descriptorPool, flags); }
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(reset_descriptor_pool) = icd.reset_descriptor_pool { return reset_descriptor_pool(device, descriptorPool, flags); }
    }
    VkResult::ErrorInitializationFailed
}

/// Allocate descriptor sets
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. pAllocateInfo points to a valid VkDescriptorSetAllocateInfo structure
// 3. pDescriptorSets points to an array with space for descriptorSetCount handles
// 4. The descriptor pool has sufficient space for the requested sets
// 5. All descriptor set layouts in pAllocateInfo are valid
// 6. The descriptor pool was not created with FREE_DESCRIPTOR_SET_BIT if individual freeing is needed
#[no_mangle]
pub unsafe extern "C" fn vkAllocateDescriptorSets(
    device: VkDevice,
    pAllocateInfo: *const VkDescriptorSetAllocateInfo,
    pDescriptorSets: *mut VkDescriptorSet,
) -> VkResult {
    if device.is_null() || pAllocateInfo.is_null() || pDescriptorSets.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    if let Some(icd) = icd_loader::icd_for_device(device) {
        if let Some(f) = icd.allocate_descriptor_sets { return f(device, pAllocateInfo, pDescriptorSets); }
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(allocate_descriptor_sets) = icd.allocate_descriptor_sets { return allocate_descriptor_sets(device, pAllocateInfo, pDescriptorSets); }
    }
    VkResult::ErrorInitializationFailed
}

/// Free descriptor sets
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. descriptorPool is a valid VkDescriptorPool
// 3. descriptorSetCount > 0 and matches the array size
// 4. pDescriptorSets points to an array of descriptorSetCount valid descriptor sets
// 5. All descriptor sets were allocated from the specified pool
// 6. The pool was created with VK_DESCRIPTOR_POOL_CREATE_FREE_DESCRIPTOR_SET_BIT
// 7. No command buffers are currently using these descriptor sets
#[no_mangle]
pub unsafe extern "C" fn vkFreeDescriptorSets(
    device: VkDevice,
    descriptorPool: VkDescriptorPool,
    descriptorSetCount: u32,
    pDescriptorSets: *const VkDescriptorSet,
) -> VkResult {
    if device.is_null() || descriptorPool.is_null() || pDescriptorSets.is_null() || descriptorSetCount == 0 {
        return VkResult::ErrorInitializationFailed;
    }
    
    if let Some(icd) = icd_loader::icd_for_device(device) {
        if let Some(f) = icd.free_descriptor_sets { return f(device, descriptorPool, descriptorSetCount, pDescriptorSets); }
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(free_descriptor_sets) = icd.free_descriptor_sets { return free_descriptor_sets(device, descriptorPool, descriptorSetCount, pDescriptorSets); }
    }
    VkResult::ErrorInitializationFailed
}

/// Update descriptor sets
// SAFETY: This function is called from C code. Caller must ensure:
// 1. device is a valid VkDevice
// 2. If descriptorWriteCount > 0, pDescriptorWrites points to valid write operations
// 3. If descriptorCopyCount > 0, pDescriptorCopies points to valid copy operations
// 4. All descriptor sets referenced in writes/copies are valid and not in use
// 5. Buffer, image, and sampler resources referenced in writes are valid
// 6. Descriptor types match the layout bindings
// 7. No command buffers are currently using the descriptor sets being updated
#[no_mangle]
pub unsafe extern "C" fn vkUpdateDescriptorSets(
    device: VkDevice,
    descriptorWriteCount: u32,
    pDescriptorWrites: *const VkWriteDescriptorSet,
    descriptorCopyCount: u32,
    pDescriptorCopies: *const VkCopyDescriptorSet,
) {
    if device.is_null() {
        return;
    }
    
    if let Some(icd) = icd_loader::icd_for_device(device) {
        if let Some(f) = icd.update_descriptor_sets { f(device, descriptorWriteCount, pDescriptorWrites, descriptorCopyCount, pDescriptorCopies); }
        return;
    }
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(update_descriptor_sets) = icd.update_descriptor_sets {
            update_descriptor_sets(device, descriptorWriteCount, pDescriptorWrites, descriptorCopyCount, pDescriptorCopies);
        }
    }
}
