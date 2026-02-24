> [!CAUTION] Documentation credibility note
> Quantified performance and benchmark claims in this repository history are in recovery and should not be treated as current production facts until revalidated under the Kronos-first flow.


# Kronos vs Standard Vulkan: Performance Comparison

## Key Finding

**Kronos forwards API calls to the same Vulkan driver**, so raw GPU performance should stay aligned with the driver. Current performance claims are framed around API-layer behavior and need revalidation against fresh workloads.

## Actual Performance Comparison

### What We Measured

Since Kronos uses the same underlying Vulkan driver, we're comparing:
- **Kronos API layer** → Vulkan ICD driver
- **Standard Vulkan loader** → Same Vulkan ICD driver

### Performance Results

| Operation | Performance | Notes |
|-----------|-------------|-------|
| Pure API operations | [deferred latency] | Handle creation, flag operations |
| Instance creation | [deferred latency] | Minimal forwarding overhead |
| Device enumeration | [deferred latency] | Cached after first call |
| Device creation | [deferred latency] | Driver-dominated, same for both |

### Where Kronos Provides Benefits

1. **API Layer Efficiency**
   - Zero-cost Rust abstractions
   - Inline optimizations
   - No virtual function overhead
   
2. **Reduced Memory Footprint (staged)**
   ```
   VkPhysicalDeviceFeatures:
   - Standard Vulkan: [deferred size]
   - Kronos: [deferred size] (targeted delta)
   ```

3. **O(1) Memory Type Lookups**
   ```
   Finding memory type with HOST_VISIBLE | HOST_COHERENT:
   - Standard: O(n) search through all types
   - Kronos: O(1) direct cache access
   ```

4. **No Graphics Overhead**
   - No swapchain management
   - No render pass setup
   - No image layout transitions
   - No graphics pipeline state

## Real-World Impact

### For Compute Workloads

1. **Application Startup**: staged target (20-30% range)
   - No graphics subsystem initialization
   - Smaller binary size
   - Fewer dependencies

2. **Memory Allocation**: staged target range (type-selection improvement)
   - Critical for frequent allocations
   - Reduces stalls in allocation-heavy workloads

3. **API Call Overhead**: staged target range
   - Matters for high-frequency operations
   - Benefits command buffer recording

### Example: ML Training Workload

For a typical machine learning training loop:
```
Per iteration:
- [deferred count] buffer allocations: [deferred latency] with O(1) lookups
- [deferred count] dispatch calls: [deferred latency] in API overhead
- [deferred count] flag checks: [deferred latency]

Total: [deferred latency] per iteration (staged)
Over [deferred count] iterations: [deferred latency] (staged)
```

## Why Not Always Faster?

1. **Driver Operations**: Device creation, shader compilation, and memory allocation are handled by the driver - Kronos can't make these faster

2. **I/O Bound Operations**: Data transfers, synchronization, and actual compute work are unchanged

3. **One-time Costs**: Benefits are most visible in initialization and high-frequency API calls

## Conclusion

Kronos is **not a faster GPU driver** - it's a **more efficient API layer** for compute workloads:

✅ **Observed Benefits (staged):**
- Targeted initialization gains
- Memory footprint reduction target
- O(1) memory type lookup target
- Reduced API call overhead target

❌ **Unchanged:**
- GPU compute performance
- Memory bandwidth
- Shader execution speed

**Best Use Cases:**
- High-frequency compute dispatches
- Applications with many buffer allocations
- Compute-only workloads
- Embedded systems with memory constraints

**Not Beneficial For:**
- Graphics applications
- Single large compute kernels
- I/O bound workloads

The ~20-30% and 5-15% values are staged snapshots and are only placeholders for revalidation under current runtime conditions.
