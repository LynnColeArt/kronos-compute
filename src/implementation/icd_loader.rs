//! Vulkan ICD (Installable Client Driver) loader
//! 
//! Loads real Vulkan drivers and forwards compute calls

use std::ffi::{CStr, CString};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use libc::{c_void, c_char};
use crate::sys::*;
use crate::core::{VkBufferCopy, VkDescriptorPoolResetFlags};
use crate::ffi::*;

/// ICD discovery paths
const ICD_SEARCH_PATHS: &[&str] = &[
    "/usr/share/vulkan/icd.d",
    "/usr/local/share/vulkan/icd.d",
    "/etc/vulkan/icd.d",
    "/usr/share/vulkan/implicit_layer.d",
];

/// Loaded ICD information
pub struct LoadedICD {
    pub library_path: PathBuf,
    pub handle: *mut c_void,
    pub api_version: u32,
    
    // Core function pointers
    pub vk_get_instance_proc_addr: PFN_vkGetInstanceProcAddr,
    
    // Instance functions
    pub create_instance: PFN_vkCreateInstance,
    pub destroy_instance: PFN_vkDestroyInstance,
    pub enumerate_physical_devices: PFN_vkEnumeratePhysicalDevices,
    pub get_physical_device_properties: PFN_vkGetPhysicalDeviceProperties,
    pub get_physical_device_queue_family_properties: PFN_vkGetPhysicalDeviceQueueFamilyProperties,
    pub get_physical_device_memory_properties: PFN_vkGetPhysicalDeviceMemoryProperties,
    
    // Device functions
    pub create_device: PFN_vkCreateDevice,
    pub destroy_device: PFN_vkDestroyDevice,
    pub get_device_proc_addr: PFN_vkGetDeviceProcAddr,
    pub get_device_queue: PFN_vkGetDeviceQueue,
    
    // Queue functions
    pub queue_submit: PFN_vkQueueSubmit,
    pub queue_wait_idle: PFN_vkQueueWaitIdle,
    pub device_wait_idle: PFN_vkDeviceWaitIdle,
    
    // Memory functions
    pub allocate_memory: PFN_vkAllocateMemory,
    pub free_memory: PFN_vkFreeMemory,
    pub map_memory: PFN_vkMapMemory,
    pub unmap_memory: PFN_vkUnmapMemory,
    
    // Buffer functions
    pub create_buffer: PFN_vkCreateBuffer,
    pub destroy_buffer: PFN_vkDestroyBuffer,
    pub get_buffer_memory_requirements: PFN_vkGetBufferMemoryRequirements,
    pub bind_buffer_memory: PFN_vkBindBufferMemory,
    
    // Descriptor functions
    pub create_descriptor_set_layout: PFN_vkCreateDescriptorSetLayout,
    pub destroy_descriptor_set_layout: PFN_vkDestroyDescriptorSetLayout,
    pub create_descriptor_pool: PFN_vkCreateDescriptorPool,
    pub destroy_descriptor_pool: PFN_vkDestroyDescriptorPool,
    pub reset_descriptor_pool: Option<unsafe extern "C" fn(VkDevice, VkDescriptorPool, VkDescriptorPoolResetFlags) -> VkResult>,
    pub allocate_descriptor_sets: PFN_vkAllocateDescriptorSets,
    pub free_descriptor_sets: Option<unsafe extern "C" fn(VkDevice, VkDescriptorPool, u32, *const VkDescriptorSet) -> VkResult>,
    pub update_descriptor_sets: PFN_vkUpdateDescriptorSets,
    
    // Pipeline functions
    pub create_pipeline_layout: PFN_vkCreatePipelineLayout,
    pub destroy_pipeline_layout: PFN_vkDestroyPipelineLayout,
    pub create_compute_pipelines: PFN_vkCreateComputePipelines,
    pub destroy_pipeline: PFN_vkDestroyPipeline,
    
    // Shader functions
    pub create_shader_module: PFN_vkCreateShaderModule,
    pub destroy_shader_module: PFN_vkDestroyShaderModule,
    
    // Command buffer functions
    pub create_command_pool: PFN_vkCreateCommandPool,
    pub destroy_command_pool: PFN_vkDestroyCommandPool,
    pub allocate_command_buffers: PFN_vkAllocateCommandBuffers,
    pub free_command_buffers: Option<unsafe extern "C" fn(VkDevice, VkCommandPool, u32, *const VkCommandBuffer)>,
    pub begin_command_buffer: PFN_vkBeginCommandBuffer,
    pub end_command_buffer: PFN_vkEndCommandBuffer,
    pub cmd_bind_pipeline: PFN_vkCmdBindPipeline,
    pub cmd_bind_descriptor_sets: PFN_vkCmdBindDescriptorSets,
    pub cmd_dispatch: PFN_vkCmdDispatch,
    pub cmd_dispatch_indirect: Option<unsafe extern "C" fn(VkCommandBuffer, VkBuffer, VkDeviceSize)>,
    pub cmd_pipeline_barrier: PFN_vkCmdPipelineBarrier,
    pub cmd_copy_buffer: Option<unsafe extern "C" fn(VkCommandBuffer, VkBuffer, VkBuffer, u32, *const VkBufferCopy)>,
    
    // Sync functions
    pub create_fence: PFN_vkCreateFence,
    pub destroy_fence: PFN_vkDestroyFence,
    pub reset_fences: PFN_vkResetFences,
    pub get_fence_status: PFN_vkGetFenceStatus,
    pub wait_for_fences: PFN_vkWaitForFences,
    pub create_semaphore: PFN_vkCreateSemaphore,
    pub destroy_semaphore: PFN_vkDestroySemaphore,
    pub create_event: PFN_vkCreateEvent,
    pub destroy_event: PFN_vkDestroyEvent,
    pub get_event_status: PFN_vkGetEventStatus,
    pub set_event: PFN_vkSetEvent,
    pub reset_event: PFN_vkResetEvent,
    pub cmd_set_event: PFN_vkCmdSetEvent,
    pub cmd_reset_event: PFN_vkCmdResetEvent,
    pub cmd_wait_events: PFN_vkCmdWaitEvents,
}

// LoadedICD is safe to send between threads because:
// 1. The library handle is only used for dlclose in Drop
// 2. Function pointers are immutable once loaded
// 3. PathBuf is already Send+Sync
unsafe impl Send for LoadedICD {}
unsafe impl Sync for LoadedICD {}

/// ICD manifest structure
#[derive(Debug)]
struct ICDManifest {
    file_format_version: String,
    library_path: String,
    api_version: Option<String>,
}

/// Global ICD loader state
lazy_static::lazy_static! {
    pub static ref ICD_LOADER: Mutex<Option<LoadedICD>> = Mutex::new(None);
}

/// Find and load Vulkan ICDs
pub fn discover_icds() -> Vec<PathBuf> {
    let mut icd_files = Vec::new();
    
    // Check environment variable first
    if let Ok(icd_filenames) = env::var("VK_ICD_FILENAMES") {
        for path in icd_filenames.split(':') {
            icd_files.push(PathBuf::from(path));
        }
    }
    
    // Search standard paths
    for search_path in ICD_SEARCH_PATHS {
        if let Ok(entries) = fs::read_dir(search_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    icd_files.push(path);
                }
            }
        }
    }
    
    icd_files
}

/// Parse ICD manifest JSON
fn parse_icd_manifest(path: &Path) -> Option<ICDManifest> {
    let content = fs::read_to_string(path).ok()?;
    
    // Simple JSON parsing for ICD manifest
    // In production, use serde_json
    let mut manifest = ICDManifest {
        file_format_version: String::new(),
        library_path: String::new(),
        api_version: None,
    };
    
    // Extract library path (hacky but works for simple manifests)
    if let Some(start) = content.find("\"library_path\"") {
        if let Some(colon) = content[start..].find(':') {
            let value_start = start + colon + 1;
            if let Some(quote1) = content[value_start..].find('"') {
                let path_start = value_start + quote1 + 1;
                if let Some(quote2) = content[path_start..].find('"') {
                    manifest.library_path = content[path_start..path_start + quote2].to_string();
                }
            }
        }
    }
    
    if manifest.library_path.is_empty() {
        return None;
    }
    
    Some(manifest)
}

/// Load an ICD library
pub fn load_icd(library_path: &Path) -> Result<LoadedICD, String> {
    unsafe {
        // Load the library
        let lib_cstr = CString::new(library_path.as_os_str().as_bytes())
            .map_err(|_| "Invalid library path")?;
        
        let handle = libc::dlopen(lib_cstr.as_ptr(), libc::RTLD_NOW | libc::RTLD_LOCAL);
        if handle.is_null() {
            let error = CStr::from_ptr(libc::dlerror()).to_string_lossy();
            return Err(format!("Failed to load library: {}", error));
        }
        
        // Get vkGetInstanceProcAddr
        let get_instance_proc_addr_name = CString::new("vkGetInstanceProcAddr").unwrap();
        let get_instance_proc_addr_ptr = libc::dlsym(handle, get_instance_proc_addr_name.as_ptr());
        
        if get_instance_proc_addr_ptr.is_null() {
            libc::dlclose(handle);
            return Err("Failed to find vkGetInstanceProcAddr".to_string());
        }
        
        let vk_get_instance_proc_addr: PFN_vkGetInstanceProcAddr = 
            std::mem::transmute(get_instance_proc_addr_ptr);
        
        // Get global functions
        let mut icd = LoadedICD {
            library_path: library_path.to_owned(),
            handle,
            api_version: VK_API_VERSION_1_0,
            vk_get_instance_proc_addr,
            create_instance: None,
            destroy_instance: None,
            enumerate_physical_devices: None,
            get_physical_device_properties: None,
            get_physical_device_queue_family_properties: None,
            get_physical_device_memory_properties: None,
            create_device: None,
            destroy_device: None,
            get_device_proc_addr: None,
            get_device_queue: None,
            queue_submit: None,
            queue_wait_idle: None,
            device_wait_idle: None,
            allocate_memory: None,
            free_memory: None,
            map_memory: None,
            unmap_memory: None,
            create_buffer: None,
            destroy_buffer: None,
            get_buffer_memory_requirements: None,
            bind_buffer_memory: None,
            create_descriptor_set_layout: None,
            destroy_descriptor_set_layout: None,
            create_descriptor_pool: None,
            destroy_descriptor_pool: None,
            reset_descriptor_pool: None,
            allocate_descriptor_sets: None,
            free_descriptor_sets: None,
            update_descriptor_sets: None,
            create_pipeline_layout: None,
            destroy_pipeline_layout: None,
            create_compute_pipelines: None,
            destroy_pipeline: None,
            create_shader_module: None,
            destroy_shader_module: None,
            create_command_pool: None,
            destroy_command_pool: None,
            allocate_command_buffers: None,
            free_command_buffers: None,
            begin_command_buffer: None,
            end_command_buffer: None,
            cmd_bind_pipeline: None,
            cmd_bind_descriptor_sets: None,
            cmd_dispatch: None,
            cmd_dispatch_indirect: None,
            cmd_pipeline_barrier: None,
            cmd_copy_buffer: None,
            create_fence: None,
            destroy_fence: None,
            reset_fences: None,
            get_fence_status: None,
            wait_for_fences: None,
            create_semaphore: None,
            destroy_semaphore: None,
            create_event: None,
            destroy_event: None,
            get_event_status: None,
            set_event: None,
            reset_event: None,
            cmd_set_event: None,
            cmd_reset_event: None,
            cmd_wait_events: None,
        };
        
        // Load global functions
        load_global_functions(&mut icd);
        
        Ok(icd)
    }
}

/// Load global function pointers
unsafe fn load_global_functions(icd: &mut LoadedICD) {
    let get_proc_addr = icd.vk_get_instance_proc_addr.expect("vkGetInstanceProcAddr should be loaded");
    
    // Helper macro to load functions
    macro_rules! load_fn {
        ($name:ident, $fn_name:expr) => {
            let name = CString::new($fn_name).unwrap();
            if let Some(addr) = get_proc_addr(VkInstance::NULL, name.as_ptr()) {
                icd.$name = std::mem::transmute(addr);
            }
        };
    }
    
    // Load instance creation functions
    load_fn!(create_instance, "vkCreateInstance");
    load_fn!(enumerate_physical_devices, "vkEnumeratePhysicalDevices");
}

/// Load instance-level functions
pub unsafe fn load_instance_functions(icd: &mut LoadedICD, instance: VkInstance) {
    let get_proc_addr = icd.vk_get_instance_proc_addr.expect("vkGetInstanceProcAddr should be loaded");
    
    macro_rules! load_fn {
        ($name:ident, $fn_name:expr) => {
            let name = CString::new($fn_name).unwrap();
            if let Some(addr) = get_proc_addr(instance, name.as_ptr()) {
                icd.$name = std::mem::transmute(addr);
            }
        };
    }
    
    // Load instance functions
    load_fn!(destroy_instance, "vkDestroyInstance");
    load_fn!(get_physical_device_properties, "vkGetPhysicalDeviceProperties");
    load_fn!(get_physical_device_queue_family_properties, "vkGetPhysicalDeviceQueueFamilyProperties");
    load_fn!(get_physical_device_memory_properties, "vkGetPhysicalDeviceMemoryProperties");
    load_fn!(create_device, "vkCreateDevice");
    load_fn!(get_device_proc_addr, "vkGetDeviceProcAddr");
}

/// Load device-level functions
pub unsafe fn load_device_functions(icd: &mut LoadedICD, device: VkDevice) {
    // Helper function to get proc address
    let get_proc_addr_helper = |name: *const c_char| -> PFN_vkVoidFunction {
        if let Some(get_device_proc_fn) = icd.get_device_proc_addr {
            get_device_proc_fn(device, name)
        } else {
            // Fall back to instance proc addr
            let instance = VkInstance::NULL; // We'd need to track this
            let get_instance_proc = icd.vk_get_instance_proc_addr.expect("vkGetInstanceProcAddr should be loaded");
            get_instance_proc(instance, name)
        }
    };
    
    macro_rules! load_fn {
        ($name:ident, $fn_name:expr) => {
            let name = CString::new($fn_name).unwrap();
            if let Some(addr) = get_proc_addr_helper(name.as_ptr()) {
                icd.$name = std::mem::transmute(addr);
            }
        };
    }
    
    // Device functions
    load_fn!(destroy_device, "vkDestroyDevice");
    load_fn!(get_device_queue, "vkGetDeviceQueue");
    load_fn!(device_wait_idle, "vkDeviceWaitIdle");
    
    // Queue functions
    load_fn!(queue_submit, "vkQueueSubmit");
    load_fn!(queue_wait_idle, "vkQueueWaitIdle");
    
    // Memory functions
    load_fn!(allocate_memory, "vkAllocateMemory");
    load_fn!(free_memory, "vkFreeMemory");
    load_fn!(map_memory, "vkMapMemory");
    load_fn!(unmap_memory, "vkUnmapMemory");
    
    // Buffer functions
    load_fn!(create_buffer, "vkCreateBuffer");
    load_fn!(destroy_buffer, "vkDestroyBuffer");
    load_fn!(get_buffer_memory_requirements, "vkGetBufferMemoryRequirements");
    load_fn!(bind_buffer_memory, "vkBindBufferMemory");
    
    // Compute-specific functions
    load_fn!(create_descriptor_set_layout, "vkCreateDescriptorSetLayout");
    load_fn!(destroy_descriptor_set_layout, "vkDestroyDescriptorSetLayout");
    load_fn!(create_descriptor_pool, "vkCreateDescriptorPool");
    load_fn!(destroy_descriptor_pool, "vkDestroyDescriptorPool");
    load_fn!(reset_descriptor_pool, "vkResetDescriptorPool");
    load_fn!(allocate_descriptor_sets, "vkAllocateDescriptorSets");
    load_fn!(free_descriptor_sets, "vkFreeDescriptorSets");
    load_fn!(update_descriptor_sets, "vkUpdateDescriptorSets");
    
    load_fn!(create_pipeline_layout, "vkCreatePipelineLayout");
    load_fn!(destroy_pipeline_layout, "vkDestroyPipelineLayout");
    load_fn!(create_compute_pipelines, "vkCreateComputePipelines");
    load_fn!(destroy_pipeline, "vkDestroyPipeline");
    
    load_fn!(create_shader_module, "vkCreateShaderModule");
    load_fn!(destroy_shader_module, "vkDestroyShaderModule");
    
    load_fn!(create_command_pool, "vkCreateCommandPool");
    load_fn!(destroy_command_pool, "vkDestroyCommandPool");
    load_fn!(allocate_command_buffers, "vkAllocateCommandBuffers");
    load_fn!(free_command_buffers, "vkFreeCommandBuffers");
    load_fn!(begin_command_buffer, "vkBeginCommandBuffer");
    load_fn!(end_command_buffer, "vkEndCommandBuffer");
    
    load_fn!(cmd_bind_pipeline, "vkCmdBindPipeline");
    load_fn!(cmd_bind_descriptor_sets, "vkCmdBindDescriptorSets");
    load_fn!(cmd_dispatch, "vkCmdDispatch");
    load_fn!(cmd_dispatch_indirect, "vkCmdDispatchIndirect");
    load_fn!(cmd_pipeline_barrier, "vkCmdPipelineBarrier");
    load_fn!(cmd_copy_buffer, "vkCmdCopyBuffer");
    
    // Sync functions
    load_fn!(create_fence, "vkCreateFence");
    load_fn!(destroy_fence, "vkDestroyFence");
    load_fn!(reset_fences, "vkResetFences");
    load_fn!(get_fence_status, "vkGetFenceStatus");
    load_fn!(wait_for_fences, "vkWaitForFences");
    
    load_fn!(create_semaphore, "vkCreateSemaphore");
    load_fn!(destroy_semaphore, "vkDestroySemaphore");
    
    load_fn!(create_event, "vkCreateEvent");
    load_fn!(destroy_event, "vkDestroyEvent");
    load_fn!(get_event_status, "vkGetEventStatus");
    load_fn!(set_event, "vkSetEvent");
    load_fn!(reset_event, "vkResetEvent");
    
    load_fn!(cmd_set_event, "vkCmdSetEvent");
    load_fn!(cmd_reset_event, "vkCmdResetEvent");
    load_fn!(cmd_wait_events, "vkCmdWaitEvents");
}

/// Initialize the ICD loader
pub fn initialize_icd_loader() -> Result<(), String> {
    let icd_files = discover_icds();
    
    if icd_files.is_empty() {
        return Err("No Vulkan ICDs found".to_string());
    }
    
    // Try to load each ICD
    for icd_file in icd_files {
        if let Some(manifest) = parse_icd_manifest(&icd_file) {
            let lib_path = if manifest.library_path.starts_with('/') {
                PathBuf::from(&manifest.library_path)
            } else {
                // Relative to manifest file
                icd_file.parent().unwrap().join(&manifest.library_path)
            };
            
            match load_icd(&lib_path) {
                Ok(icd) => {
                    println!("Loaded ICD: {:?}", lib_path);
                    *ICD_LOADER.lock().unwrap() = Some(icd);
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("Failed to load ICD {:?}: {}", lib_path, e);
                }
            }
        }
    }
    
    Err("Failed to load any Vulkan ICD".to_string())
}

/// Get the loaded ICD
pub fn get_icd() -> Option<&'static LoadedICD> {
    unsafe {
        ICD_LOADER.lock().unwrap().as_ref().map(|icd| {
            &*(icd as *const LoadedICD)
        })
    }
}

use std::sync::Mutex;