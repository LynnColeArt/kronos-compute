//! Timeline semaphore structures for Kronos

use std::ffi::c_void;
use std::ptr;
use crate::sys::*;
use crate::core::enums::*;
use crate::core::flags::*;

/// Semaphore type create info (for timeline semaphores)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkSemaphoreTypeCreateInfo {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub semaphoreType: VkSemaphoreType,
    pub initialValue: u64,
}

impl Default for VkSemaphoreTypeCreateInfo {
    fn default() -> Self {
        Self {
            sType: VkStructureType::SemaphoreTypeCreateInfo,
            pNext: ptr::null(),
            semaphoreType: VkSemaphoreType::Binary,
            initialValue: 0,
        }
    }
}

/// Timeline semaphore submit info
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkTimelineSemaphoreSubmitInfo {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub waitSemaphoreValueCount: u32,
    pub pWaitSemaphoreValues: *const u64,
    pub signalSemaphoreValueCount: u32,
    pub pSignalSemaphoreValues: *const u64,
}

impl Default for VkTimelineSemaphoreSubmitInfo {
    fn default() -> Self {
        Self {
            sType: VkStructureType::TimelineSemaphoreSubmitInfo,
            pNext: ptr::null(),
            waitSemaphoreValueCount: 0,
            pWaitSemaphoreValues: ptr::null(),
            signalSemaphoreValueCount: 0,
            pSignalSemaphoreValues: ptr::null(),
        }
    }
}

/// Semaphore wait info for timeline semaphores
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VkSemaphoreWaitInfo {
    pub sType: VkStructureType,
    pub pNext: *const c_void,
    pub flags: VkSemaphoreWaitFlags,
    pub semaphoreCount: u32,
    pub pSemaphores: *const VkSemaphore,
    pub pValues: *const u64,
}

impl Default for VkSemaphoreWaitInfo {
    fn default() -> Self {
        Self {
            sType: VkStructureType::SemaphoreWaitInfo,
            pNext: ptr::null(),
            flags: VkSemaphoreWaitFlags::empty(),
            semaphoreCount: 0,
            pSemaphores: ptr::null(),
            pValues: ptr::null(),
        }
    }
}