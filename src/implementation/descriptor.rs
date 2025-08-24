//! Descriptor set management implementation

use crate::sys::*;
use crate::core::*;
use crate::ffi::*;

/// Create descriptor set layout
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
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(create_descriptor_set_layout) = icd.create_descriptor_set_layout {
            return create_descriptor_set_layout(device, pCreateInfo, pAllocator, pSetLayout);
        }
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}

/// Destroy descriptor set layout
#[no_mangle]
pub unsafe extern "C" fn vkDestroyDescriptorSetLayout(
    device: VkDevice,
    descriptorSetLayout: VkDescriptorSetLayout,
    pAllocator: *const VkAllocationCallbacks,
) {
    if device.is_null() || descriptorSetLayout.is_null() {
        return;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(destroy_descriptor_set_layout) = icd.destroy_descriptor_set_layout {
            destroy_descriptor_set_layout(device, descriptorSetLayout, pAllocator);
        }
    }
}

/// Create descriptor pool
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
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(create_descriptor_pool) = icd.create_descriptor_pool {
            return create_descriptor_pool(device, pCreateInfo, pAllocator, pDescriptorPool);
        }
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}

/// Destroy descriptor pool
#[no_mangle]
pub unsafe extern "C" fn vkDestroyDescriptorPool(
    device: VkDevice,
    descriptorPool: VkDescriptorPool,
    pAllocator: *const VkAllocationCallbacks,
) {
    if device.is_null() || descriptorPool.is_null() {
        return;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(destroy_descriptor_pool) = icd.destroy_descriptor_pool {
            destroy_descriptor_pool(device, descriptorPool, pAllocator);
        }
    }
}

/// Reset descriptor pool
#[no_mangle]
pub unsafe extern "C" fn vkResetDescriptorPool(
    device: VkDevice,
    descriptorPool: VkDescriptorPool,
    flags: VkDescriptorPoolResetFlags,
) -> VkResult {
    if device.is_null() || descriptorPool.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(reset_descriptor_pool) = icd.reset_descriptor_pool {
            return reset_descriptor_pool(device, descriptorPool, flags);
        }
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}

/// Allocate descriptor sets
#[no_mangle]
pub unsafe extern "C" fn vkAllocateDescriptorSets(
    device: VkDevice,
    pAllocateInfo: *const VkDescriptorSetAllocateInfo,
    pDescriptorSets: *mut VkDescriptorSet,
) -> VkResult {
    if device.is_null() || pAllocateInfo.is_null() || pDescriptorSets.is_null() {
        return VkResult::ErrorInitializationFailed;
    }
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(allocate_descriptor_sets) = icd.allocate_descriptor_sets {
            return allocate_descriptor_sets(device, pAllocateInfo, pDescriptorSets);
        }
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}

/// Free descriptor sets
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
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(free_descriptor_sets) = icd.free_descriptor_sets {
            return free_descriptor_sets(device, descriptorPool, descriptorSetCount, pDescriptorSets);
        }
    }
    
    // No ICD available
    VkResult::ErrorInitializationFailed
}

/// Update descriptor sets
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
    
    // Forward to real ICD
    if let Some(icd) = super::forward::get_icd_if_enabled() {
        if let Some(update_descriptor_sets) = icd.update_descriptor_sets {
            update_descriptor_sets(device, descriptorWriteCount, pDescriptorWrites,
                                 descriptorCopyCount, pDescriptorCopies);
        }
    }
}