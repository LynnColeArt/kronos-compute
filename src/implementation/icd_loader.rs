//! Vulkan ICD (Installable Client Driver) loader
//! 
//! Loads real Vulkan drivers and forwards compute calls

use std::ffi::{CStr, CString};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use libc::{c_void, c_char};
use std::sync::{Arc, Mutex};
use log::{info, warn, debug};
use serde::{Deserialize, Serialize};
use crate::sys::*;
use crate::core::*;
use crate::ffi::*;
use super::error::IcdError;

/// Get platform-specific ICD search paths
fn get_icd_search_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    // Check environment variable first (cross-platform override)
    if let Ok(custom_paths) = env::var("KRONOS_ICD_SEARCH_PATHS") {
        for path in custom_paths.split(if cfg!(windows) { ';' } else { ':' }) {
            paths.push(PathBuf::from(path));
        }
        return paths;
    }
    
    // Platform-specific default paths
    #[cfg(target_os = "linux")]
    {
        paths.extend([
            PathBuf::from("/usr/share/vulkan/icd.d"),
            PathBuf::from("/usr/local/share/vulkan/icd.d"),
            PathBuf::from("/etc/vulkan/icd.d"),
            PathBuf::from("/usr/share/vulkan/implicit_layer.d"),
        ]);
    }
    
    #[cfg(target_os = "windows")]
    {
        // Windows registry paths - simplified for now
        if let Ok(system_root) = env::var("SYSTEMROOT") {
            paths.push(PathBuf::from(system_root).join("System32").join("vulkan"));
        }
        paths.push(PathBuf::from("C:\\Windows\\System32\\vulkan"));
        
        // Program Files paths
        if let Ok(program_files) = env::var("PROGRAMFILES") {
            paths.push(PathBuf::from(program_files).join("Vulkan").join("Config"));
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        paths.extend([
            PathBuf::from("/usr/local/share/vulkan/icd.d"),
            PathBuf::from("/System/Library/Extensions"),
            PathBuf::from("/Library/Extensions"),
        ]);
        
        // User-specific paths
        if let Ok(home) = env::var("HOME") {
            paths.push(PathBuf::from(home).join(".local/share/vulkan/icd.d"));
        }
    }
    
    // Canonicalize where possible to mitigate traversal issues
    let mut canon = Vec::new();
    for p in paths {
        match fs::canonicalize(&p) {
            Ok(cp) => canon.push(cp),
            Err(_) => canon.push(p), // keep original if canonicalize fails
        }
    }
    log::info!("ICD search paths: {:#?}", canon);
    canon
}

/// Loaded ICD information
#[derive(Clone)]
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
    pub cmd_push_constants: Option<unsafe extern "C" fn(VkCommandBuffer, VkPipelineLayout, VkShaderStageFlags, u32, u32, *const c_void)>,
    
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
    
    // Timeline semaphore functions
    pub wait_semaphores: Option<unsafe extern "C" fn(VkDevice, *const VkSemaphoreWaitInfo, u64) -> VkResult>,
}

// SAFETY: LoadedICD is safe to send between threads because:
// 1. The library handle is only used for dlclose in Drop
// 2. Function pointers are immutable once loaded
// 3. PathBuf is already Send+Sync
// 4. No mutable state is shared between threads
unsafe impl Send for LoadedICD {}
unsafe impl Sync for LoadedICD {}

/// Public info about a loadable ICD
#[derive(Debug, Clone)]
pub struct IcdInfo {
    pub library_path: PathBuf,
    pub manifest_path: Option<PathBuf>,
    pub api_version: u32,
    pub is_software: bool,
}

/// ICD manifest root structure
#[derive(Debug, Deserialize, Serialize)]
struct ICDManifestRoot {
    file_format_version: String,
    #[serde(rename = "ICD")]
    icd: ICDManifest,
}

/// ICD manifest structure
#[derive(Debug, Deserialize, Serialize)]
struct ICDManifest {
    library_path: String,
    api_version: Option<String>,
}

lazy_static::lazy_static! {
    // Global ICD loader state (Arc allows safe sharing; we replace on updates)
    pub static ref ICD_LOADER: Mutex<Option<Arc<LoadedICD>>> = Mutex::new(None);
}

/// Find and load Vulkan ICDs
pub fn discover_icds() -> Vec<PathBuf> {
    let mut icd_files = Vec::new();
    let mut env_icds = Vec::new();
    
    // Check environment variable - these will be prioritized but not exclusive
    if let Ok(icd_filenames) = env::var("VK_ICD_FILENAMES") {
        let separator = if cfg!(windows) { ';' } else { ':' };
        for path in icd_filenames.split(separator) {
            let raw = PathBuf::from(path);
            let can = fs::canonicalize(&raw).unwrap_or(raw);
            if can.exists() {
                env_icds.push(can.clone());
                icd_files.push(can);
            } else {
                warn!("VK_ICD_FILENAMES contains non-existent path: {}", path);
            }
        }
        if !env_icds.is_empty() {
            info!("Found {} ICD files from VK_ICD_FILENAMES (will be prioritized)", env_icds.len());
        }
    }
    
    // Always search platform-specific paths for all available ICDs
    let search_paths = get_icd_search_paths();
    for search_path in &search_paths {
        if let Ok(entries) = fs::read_dir(search_path) {
            let mut path_count = 0;
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    // Skip if already added from environment variable
                    if !env_icds.contains(&path) {
                        let can = fs::canonicalize(&path).unwrap_or(path);
                        log::debug!("Discovered ICD candidate: {}", can.display());
                        icd_files.push(can);
                        path_count += 1;
                    }
                }
            }
            if path_count > 0 {
                info!("Found {} additional ICD manifest files in {}", path_count, search_path.display());
            }
        }
    }
    
    if icd_files.is_empty() {
        warn!("No ICD manifest files found in any search paths: {:#?}", search_paths);
    }
    
    icd_files
}

/// Parse ICD manifest JSON
fn parse_icd_manifest(path: &Path) -> Option<ICDManifest> {
    let content = fs::read_to_string(path).ok()?;
    
    // Parse JSON using serde_json
    match serde_json::from_str::<ICDManifestRoot>(&content) {
        Ok(manifest_root) => {
            if manifest_root.icd.library_path.is_empty() {
                warn!("ICD manifest has empty library_path: {}", path.display());
                return None;
            }
            debug!("Successfully parsed ICD manifest: {} -> {}", path.display(), manifest_root.icd.library_path);
            Some(manifest_root.icd)
        }
        Err(e) => {
            warn!("Failed to parse ICD manifest {}: {}", path.display(), e);
            None
        }
    }
}

/// Parse API version from manifest string like "1.3.268" into VK_MAKE_VERSION
fn parse_api_version(version: &str) -> Option<u32> {
    let mut parts = version.split('.');
    let major = parts.next()?.parse::<u32>().ok()?;
    let minor = parts.next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
    let patch = parts.next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
    Some(VK_MAKE_VERSION(major, minor, patch))
}

/// Return all loadable ICDs with metadata (does not mutate global state)
pub fn available_icds() -> Vec<IcdInfo> {
    let mut out = Vec::new();
    let icd_files = discover_icds();

    for icd_file in &icd_files {
        if let Some(manifest) = parse_icd_manifest(icd_file) {
            // Build candidate library paths; prefer absolute, else try as-provided and manifest-relative
            let mut candidates: Vec<PathBuf> = Vec::new();
            if Path::new(&manifest.library_path).is_absolute() {
                candidates.push(PathBuf::from(&manifest.library_path));
            } else {
                candidates.push(PathBuf::from(&manifest.library_path));
                if let Some(parent) = icd_file.parent() {
                    candidates.push(parent.join(&manifest.library_path));
                }
            }

            // Attempt to load first working candidate for this manifest
            for cand in &candidates {
                if let Ok(icd) = load_icd(cand) {
                    let path_str = icd.library_path.to_string_lossy();
                    let is_software = path_str.contains("lvp") || path_str.contains("swrast") || path_str.contains("llvmpipe");
                    let api_version = manifest
                        .api_version
                        .as_deref()
                        .and_then(parse_api_version)
                        .unwrap_or(icd.api_version);

                    out.push(IcdInfo {
                        library_path: icd.library_path,
                        manifest_path: Some(icd_file.clone()),
                        api_version,
                        is_software,
                    });
                    break; // one entry per manifest
                }
            }
        }
    }

    out
}

// Preferred ICD selection (process-wide for now)
#[derive(Debug, Clone)]
enum IcdPreference {
    Path(PathBuf),
    Index(usize),
}

lazy_static::lazy_static! {
    static ref PREFERRED_ICD: Mutex<Option<IcdPreference>> = Mutex::new(None);
}

pub fn set_preferred_icd_path<P: Into<PathBuf>>(path: P) {
    if let Ok(mut pref) = PREFERRED_ICD.lock() {
        *pref = Some(IcdPreference::Path(path.into()));
    }
}

pub fn set_preferred_icd_index(index: usize) {
    if let Ok(mut pref) = PREFERRED_ICD.lock() {
        *pref = Some(IcdPreference::Index(index));
    }
}

pub fn clear_preferred_icd() {
    if let Ok(mut pref) = PREFERRED_ICD.lock() {
        *pref = None;
    }
}

/// Get info for the currently selected/loaded ICD (if any)
pub fn selected_icd_info() -> Option<IcdInfo> {
    let icd = get_icd()?;
    let path = icd.library_path.clone();
    let path_str = path.to_string_lossy();
    let is_software = path_str.contains("lvp") || path_str.contains("swrast") || path_str.contains("llvmpipe");
    Some(IcdInfo {
        library_path: path,
        manifest_path: None,
        api_version: icd.api_version,
        is_software,
    })
}

/// Load an ICD library
fn is_trusted_library(path: &Path) -> bool {
    if env::var("KRONOS_ALLOW_UNTRUSTED_LIBS").map(|v| v == "1").unwrap_or(false) {
        return true;
    }
    #[cfg(target_os = "linux")]
    {
        const TRUSTED_PREFIXES: &[&str] = &[
            "/usr/lib",
            "/usr/lib64",
            "/usr/local/lib",
            "/lib",
            "/lib64",
            "/usr/lib/x86_64-linux-gnu",
        ];
        let p = path.to_string_lossy();
        return TRUSTED_PREFIXES.iter().any(|prefix| p.starts_with(prefix));
    }
    #[cfg(not(target_os = "linux"))]
    {
        // Conservative default on other platforms
        true
    }
}

pub fn load_icd(library_path: &Path) -> Result<LoadedICD, IcdError> {
    // SAFETY: This function uses unsafe operations for:
    // 1. dlopen/dlsym - We ensure the library path is valid and null-terminated
    // 2. Function pointer transmutation - We trust the Vulkan ICD to provide correct function signatures
    // 3. The loaded library handle is kept alive for the lifetime of LoadedICD
    unsafe {
        // Resolve and validate the library path
        let canon = fs::canonicalize(library_path).unwrap_or_else(|_| library_path.to_path_buf());
        let meta = fs::metadata(&canon)
            .map_err(|_| IcdError::LibraryLoadFailed(format!("{} (metadata not found)", canon.display())))?;
        if !meta.is_file() {
            return Err(IcdError::LibraryLoadFailed(format!("{} is not a regular file", canon.display())));
        }
        if !is_trusted_library(&canon) {
            return Err(IcdError::LibraryLoadFailed(format!(
                "{} rejected by trust policy (set KRONOS_ALLOW_UNTRUSTED_LIBS=1 to override)",
                canon.display()
            )));
        }

        // Load the library
        let lib_cstr = CString::new(canon.as_os_str().as_bytes())?;
        
        let handle = libc::dlopen(lib_cstr.as_ptr(), libc::RTLD_NOW | libc::RTLD_LOCAL);
        if handle.is_null() {
            let error = CStr::from_ptr(libc::dlerror()).to_string_lossy();
            return Err(IcdError::LibraryLoadFailed(format!("{}: {}", library_path.display(), error)));
        }
        
        // Get vk_icdGetInstanceProcAddr (ICD entry point)
        let get_instance_proc_addr_name = CString::new("vk_icdGetInstanceProcAddr")?;
        let get_instance_proc_addr_ptr = libc::dlsym(handle, get_instance_proc_addr_name.as_ptr());
        
        if get_instance_proc_addr_ptr.is_null() {
            libc::dlclose(handle);
            return Err(IcdError::MissingFunction("vk_icdGetInstanceProcAddr"));
        }
        
        let vk_get_instance_proc_addr: PFN_vkGetInstanceProcAddr = 
            std::mem::transmute(get_instance_proc_addr_ptr);
        
        // Get global functions
        let mut icd = LoadedICD {
            library_path: canon,
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
            cmd_push_constants: None,
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
            wait_semaphores: None,
        };
        
        // Load global functions and propagate failure instead of silently ignoring it
        load_global_functions_inner(&mut icd)?;
        Ok(icd)
    }
}

/// Load global function pointers
///
/// # Safety
///
/// This function is unsafe because:
/// - It calls vkGetInstanceProcAddr through a function pointer
/// - It transmutes void pointers to function pointers without type checking
/// - The caller must ensure icd contains a valid vkGetInstanceProcAddr function pointer
/// - The ICD library must be loaded and remain valid for the lifetime of icd
/// - Function signatures must match the Vulkan specification exactly
/// - Incorrect function pointers will cause undefined behavior when called
unsafe fn load_global_functions_inner(icd: &mut LoadedICD) -> Result<(), IcdError> {
    let get_proc_addr = icd.vk_get_instance_proc_addr
        .ok_or(IcdError::MissingFunction("vkGetInstanceProcAddr not loaded"))?;
    
    // Helper macro to load functions
    macro_rules! load_fn {
        ($name:ident, $fn_name:expr) => {
            // These are static strings, so they won't have null bytes
            let name = CString::new($fn_name)
                .expect(concat!("Invalid function name: ", $fn_name));
            if let Some(addr) = get_proc_addr(VkInstance::NULL, name.as_ptr()) {
                icd.$name = std::mem::transmute(addr);
            }
        };
    }
    
    // Load instance creation functions
    load_fn!(create_instance, "vkCreateInstance");
    
    debug!("Loaded global functions - create_instance: {:?}",
           icd.create_instance.is_some());
    
    Ok(())
}

/// Load instance-level functions
///
/// # Safety
///
/// This function is unsafe because:
/// - It calls vkGetInstanceProcAddr with the provided instance handle
/// - It transmutes void pointers to function pointers without type checking
/// - The instance must be a valid VkInstance created by this ICD
/// - The icd must contain valid function pointers from the same ICD that created the instance
/// - The instance must remain valid for at least as long as these functions are used
/// - Using an invalid instance handle will cause undefined behavior
/// - Function signatures must match the Vulkan specification exactly
pub unsafe fn load_instance_functions_inner(icd: &mut LoadedICD, instance: VkInstance) -> Result<(), IcdError> {
    let get_proc_addr = icd.vk_get_instance_proc_addr
        .ok_or(IcdError::MissingFunction("vkGetInstanceProcAddr not loaded"))?;
    
    macro_rules! load_fn {
        ($name:ident, $fn_name:expr) => {
            let name = CString::new($fn_name)
                .expect(concat!("Invalid function name: ", $fn_name));
            if let Some(addr) = get_proc_addr(instance, name.as_ptr()) {
                icd.$name = std::mem::transmute(addr);
            }
        };
    }
    
    // Load instance functions
    load_fn!(destroy_instance, "vkDestroyInstance");
    load_fn!(enumerate_physical_devices, "vkEnumeratePhysicalDevices");
    load_fn!(get_physical_device_properties, "vkGetPhysicalDeviceProperties");
    load_fn!(get_physical_device_queue_family_properties, "vkGetPhysicalDeviceQueueFamilyProperties");
    load_fn!(get_physical_device_memory_properties, "vkGetPhysicalDeviceMemoryProperties");
    load_fn!(create_device, "vkCreateDevice");
    load_fn!(get_device_proc_addr, "vkGetDeviceProcAddr");
    
    debug!("Loaded instance functions - enumerate_physical_devices: {:?}",
           icd.enumerate_physical_devices.is_some());
    
    Ok(())
}

/// Load device-level functions
///
/// # Safety
///
/// This function is unsafe because:
/// - It calls vkGetDeviceProcAddr or vkGetInstanceProcAddr with device/instance handles
/// - It transmutes void pointers to function pointers without type checking
/// - The device must be a valid VkDevice created by this ICD
/// - The icd must contain valid function pointers from the same ICD that created the device
/// - The device must remain valid for at least as long as these functions are used
/// - Using an invalid device handle will cause undefined behavior
/// - Function signatures must match the Vulkan specification exactly
/// - The fallback to instance proc addr requires a valid instance context
pub unsafe fn load_device_functions_inner(icd: &mut LoadedICD, device: VkDevice) -> Result<(), IcdError> {
    // Prefer device-level loader; do not fall back to NULL instance, which is invalid.
    let get_device_proc_fn = icd
        .get_device_proc_addr
        .ok_or(IcdError::MissingFunction("vkGetDeviceProcAddr not loaded"))?;
    
    // Helper function to get proc address strictly via device
    let get_proc_addr_helper = |name: *const c_char| -> PFN_vkVoidFunction {
        get_device_proc_fn(device, name)
    };
    
    macro_rules! load_fn {
        ($name:ident, $fn_name:expr) => {
            let name = CString::new($fn_name)
                .expect(concat!("Invalid function name: ", $fn_name));
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
    load_fn!(cmd_push_constants, "vkCmdPushConstants");
    
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
    
    // Timeline semaphore functions (optional)
    load_fn!(wait_semaphores, "vkWaitSemaphores");
    
    Ok(())
}

/// Initialize the ICD loader
pub fn initialize_icd_loader() -> Result<(), IcdError> {
    info!("Initializing ICD loader...");
    let icd_files = discover_icds();
    
    if icd_files.is_empty() {
        warn!("No ICD manifest files found");
        return Err(IcdError::NoManifestsFound);
    }
    
    info!("Found {} ICD manifest files", icd_files.len());
    
    // Check if we have environment variable override
    let env_icd_count = if let Ok(icd_filenames) = env::var("VK_ICD_FILENAMES") {
        let separator = if cfg!(windows) { ';' } else { ':' };
        icd_filenames.split(separator).filter(|s| !s.is_empty()).count()
    } else {
        0
    };
    
    // Collect all successfully loaded ICDs with priority flag
    let mut loaded_icds = Vec::new();
    
    // Try to load each ICD
    for (idx, icd_file) in icd_files.iter().enumerate() {
        if let Some(manifest) = parse_icd_manifest(&icd_file) {
            // Build candidate library paths. Prefer the path as provided to allow
            // dynamic linker search to resolve common locations (e.g. /usr/lib). As a
            // fallback, try relative to the manifest directory.
            let mut candidates: Vec<PathBuf> = Vec::new();
            if Path::new(&manifest.library_path).is_absolute() {
                candidates.push(PathBuf::from(&manifest.library_path));
            } else {
                // As provided (lets dlopen search LD_LIBRARY_PATH etc.)
                candidates.push(PathBuf::from(&manifest.library_path));
                // Fallback relative to manifest directory
                if let Some(parent) = icd_file.parent() {
                    candidates.push(parent.join(&manifest.library_path));
                }
            }

            let mut loaded_ok: Option<LoadedICD> = None;
            for cand in &candidates {
                // Canonicalize candidate for validation
                let can = fs::canonicalize(cand).unwrap_or(cand.clone());
                log::info!("Attempting to load ICD library: {} (from {})", can.display(), icd_file.display());
                match load_icd(&can) {
                    Ok(icd) => {
                        loaded_ok = Some(icd);
                        break;
                    }
                    Err(e) => {
                        warn!("Failed to load candidate {}: {}", can.display(), e);
                    }
                }
            }

            if let Some(icd) = loaded_ok {
                    // Check if this is a software renderer
                    let path_str = icd.library_path.to_string_lossy();
                    let is_software = path_str.contains("lvp") ||
                                     path_str.contains("swrast") ||
                                     path_str.contains("llvmpipe");
                    
                    // Environment variable ICDs are prioritized (first N entries from discover_icds)
                    let is_env_priority = idx < env_icd_count;
                    
                    let icd_type = if is_software { "software" } else { "hardware" };
                    let priority_str = if is_env_priority { " (VK_ICD_FILENAMES priority)" } else { "" };
                    info!("Successfully loaded {} Vulkan ICD: {}{}", icd_type, icd.library_path.display(), priority_str);
                    
                    loaded_icds.push((icd, is_software, is_env_priority));
            } else {
                warn!("Failed to load ICD from any candidate for manifest {}", icd_file.display());
            }
        }
    }
    
    if loaded_icds.is_empty() {
        return Err(IcdError::InvalidManifest("Failed to load any Vulkan ICD".to_string()));
    }

    // Optional policy: if any hardware ICDs are present, prefer them over software by filtering
    let prefer_hardware = env::var("KRONOS_PREFER_HARDWARE").map(|v| v != "0").unwrap_or(true);
    if prefer_hardware {
        let any_hw = loaded_icds.iter().any(|(_, is_sw, _)| !*is_sw);
        if any_hw {
            loaded_icds.retain(|(_, is_sw, _)| !*is_sw);
            info!("Hardware ICDs available; software ICDs will be ignored (set KRONOS_PREFER_HARDWARE=0 to disable)");
        }
    }

    // Sort ICDs: env priority first, then hardware (already filtered if policy), then software renderers
    loaded_icds.sort_by_key(|(_, is_software, is_env_priority)| {
        (!is_env_priority, *is_software)
    });
    
    // Log all available ICDs
    info!("Available ICDs: {} hardware, {} software", 
          loaded_icds.iter().filter(|(_, is_sw, _)| !is_sw).count(),
          loaded_icds.iter().filter(|(_, is_sw, _)| *is_sw).count());
    
    // Check for explicit preference
    let preferred = PREFERRED_ICD.lock().ok().and_then(|p| p.clone());
    let (best_icd, is_software, is_env_priority) = if let Some(pref) = preferred {
        match pref {
            IcdPreference::Path(want) => {
                if let Some((idx, _)) = loaded_icds.iter().enumerate().find(|(_, (icd, _, _))| icd.library_path == want) {
                    loaded_icds.into_iter().nth(idx).unwrap()
                } else {
                    warn!("Preferred ICD path not found: {} — falling back to default selection", want.display());
                    loaded_icds.into_iter().next().unwrap()
                }
            }
            IcdPreference::Index(i) => {
                if i < loaded_icds.len() {
                    loaded_icds.into_iter().nth(i).unwrap()
                } else {
                    warn!("Preferred ICD index {} out of range ({}); falling back to default", i, loaded_icds.len());
                    loaded_icds.into_iter().next().unwrap()
                }
            }
        }
    } else {
        // Use the best ICD (first in sorted list)
        loaded_icds.into_iter().next().unwrap()
    };
    
    if is_env_priority {
        info!("Using ICD specified by VK_ICD_FILENAMES: {}", best_icd.library_path.display());
    } else if is_software {
        warn!("Using software renderer - no hardware Vulkan drivers found");
        info!("To use hardware drivers, ensure they are installed and ICD files are in /usr/share/vulkan/icd.d/");
    } else {
        info!("Selected hardware Vulkan driver: {}", best_icd.library_path.display());
    }
    
    *ICD_LOADER.lock()? = Some(Arc::new(best_icd));
    Ok(())
}

/// Get the loaded ICD (shared clone)
pub fn get_icd() -> Option<Arc<LoadedICD>> {
    ICD_LOADER.lock().ok()?.as_ref().cloned()
}

/// Apply a mutation to the current ICD by replacing it with an updated copy
fn replace_icd<F>(mutator: F) -> Result<(), IcdError>
where
    F: FnOnce(&mut LoadedICD) -> Result<(), IcdError>,
{
    let mut guard = ICD_LOADER.lock()?;
    let current = guard.as_ref().ok_or(IcdError::NoIcdLoaded)?;
    let mut updated = (**current).clone();
    mutator(&mut updated)?;
    *guard = Some(Arc::new(updated));
    Ok(())
}

/// Update instance-level function pointers for the current ICD
pub unsafe fn update_instance_functions(instance: VkInstance) -> Result<(), IcdError> {
    replace_icd(|icd| load_instance_functions_inner(icd, instance))
}

/// Update device-level function pointers for the current ICD
pub unsafe fn update_device_functions(device: VkDevice) -> Result<(), IcdError> {
    replace_icd(|icd| load_device_functions_inner(icd, device))
}
