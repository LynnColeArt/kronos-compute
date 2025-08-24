#!/bin/bash
# Kronos Benchmark Validation Script

echo "==================================="
echo "Kronos Benchmark Validation"
echo "==================================="
echo ""

# Build
echo "Building Kronos with optimizations..."
cargo build --release --features implementation --quiet || exit 1
echo "✓ Build complete"
echo ""

# Run tests
echo "Running test suite..."
cargo test --features implementation --quiet 2>&1 | grep "test result:"
echo ""

# Run specific optimization tests
echo "Running optimization validation tests..."
echo ""

# Test 1: FFI Safety
echo "1. FFI Safety Tests:"
cargo test --test ffi_safety --features implementation --quiet -- --nocapture 2>&1 | grep -E "(ok|FAILED|✓)"

# Test 2: Performance Counters (simulated)
echo ""
echo "2. Performance Counter Tests:"
cargo test --test perf_counters --features implementation --quiet -- --nocapture 2>&1 | grep -E "(✓|✗|Expected)"

# Test 3: Library tests count
echo ""
echo "3. Total Test Count:"
TOTAL_TESTS=$(cargo test --features implementation --quiet 2>&1 | grep -E "test result:" | grep -oE "[0-9]+ passed" | grep -oE "[0-9]+" | paste -sd+ | bc)
echo "Total tests passing: $TOTAL_TESTS"

# Check examples build
echo ""
echo "4. Example Build Status:"
echo -n "  compute_simple: "
if cargo build --example compute_simple --features implementation --quiet 2>/dev/null; then
    echo "✓ Builds"
else
    echo "✗ Failed"
fi

echo -n "  icd_info: "
if cargo build --example icd_info --features implementation --quiet 2>/dev/null; then
    echo "✓ Builds"
else  
    echo "✗ Failed"
fi

# Check for ICDs
echo ""
echo "5. ICD Discovery:"
if [ -d "/usr/share/vulkan/icd.d" ]; then
    ICD_COUNT=$(ls /usr/share/vulkan/icd.d/*.json 2>/dev/null | wc -l)
    echo "  Found $ICD_COUNT ICD files"
    ls /usr/share/vulkan/icd.d/*.json 2>/dev/null | while read icd; do
        echo "    - $(basename $icd)"
    done
else
    echo "  ✗ No ICD directory found"
fi

echo ""
echo "==================================="
echo "Validation Complete"
echo "===================================" 