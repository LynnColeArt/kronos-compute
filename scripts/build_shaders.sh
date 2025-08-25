#!/bin/bash
# Build SPIR-V shaders for Kronos Compute

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
SHADER_DIR="$PROJECT_ROOT/shaders"
OUTPUT_DIR="$SHADER_DIR"

echo "Building SPIR-V shaders for Kronos Compute..."
echo "==========================================="

# Check for shader compiler
COMPILER=""
if command -v glslc &> /dev/null; then
    COMPILER="glslc"
elif command -v glslangValidator &> /dev/null; then
    COMPILER="glslangValidator"
else
    echo "Error: No SPIR-V compiler found!"
    echo "Please install either:"
    echo "  - glslc (from shaderc/Google)"
    echo "  - glslangValidator (from Vulkan SDK)"
    exit 1
fi

echo "Using compiler: $COMPILER"

# Create output directory if it doesn't exist
mkdir -p "$OUTPUT_DIR"

# Counter for processed shaders
PROCESSED=0
FAILED=0

# Find all GLSL compute shaders
for shader in "$SHADER_DIR"/*.comp; do
    if [ -f "$shader" ]; then
        BASENAME=$(basename "$shader" .comp)
        OUTPUT="$OUTPUT_DIR/${BASENAME}.spv"
        
        echo -n "Compiling $BASENAME.comp... "
        
        if [ "$COMPILER" = "glslc" ]; then
            if glslc -fshader-stage=comp "$shader" -o "$OUTPUT" 2>/dev/null; then
                echo "✓"
                ((PROCESSED++))
            else
                echo "✗"
                echo "  Error compiling $shader:"
                glslc -fshader-stage=comp "$shader" -o "$OUTPUT"
                ((FAILED++))
            fi
        else
            if glslangValidator -V "$shader" -o "$OUTPUT" 2>/dev/null; then
                echo "✓"
                ((PROCESSED++))
            else
                echo "✗"
                echo "  Error compiling $shader:"
                glslangValidator -V "$shader" -o "$OUTPUT"
                ((FAILED++))
            fi
        fi
    fi
done

# Also look for .glsl files with compute shaders
for shader in "$SHADER_DIR"/*.glsl; do
    if [ -f "$shader" ] && grep -q "#version.*\|void main" "$shader"; then
        BASENAME=$(basename "$shader" .glsl)
        OUTPUT="$OUTPUT_DIR/${BASENAME}.spv"
        
        echo -n "Compiling $BASENAME.glsl... "
        
        if [ "$COMPILER" = "glslc" ]; then
            if glslc -fshader-stage=comp "$shader" -o "$OUTPUT" 2>/dev/null; then
                echo "✓"
                ((PROCESSED++))
            else
                echo "✗"
                echo "  Error compiling $shader:"
                glslc -fshader-stage=comp "$shader" -o "$OUTPUT"
                ((FAILED++))
            fi
        else
            if glslangValidator -V -S comp "$shader" -o "$OUTPUT" 2>/dev/null; then
                echo "✓"
                ((PROCESSED++))
            else
                echo "✗"
                echo "  Error compiling $shader:"
                glslangValidator -V -S comp "$shader" -o "$OUTPUT"
                ((FAILED++))
            fi
        fi
    fi
done

echo ""
echo "Shader compilation complete!"
echo "  Processed: $PROCESSED"
echo "  Failed: $FAILED"

if [ $FAILED -gt 0 ]; then
    echo ""
    echo "Some shaders failed to compile. Please check the errors above."
    exit 1
fi

# List generated SPIR-V files
echo ""
echo "Generated SPIR-V files:"
for spv in "$OUTPUT_DIR"/*.spv; do
    if [ -f "$spv" ]; then
        SIZE=$(stat -c%s "$spv" 2>/dev/null || stat -f%z "$spv" 2>/dev/null || echo "?")
        echo "  $(basename "$spv") ($SIZE bytes)"
    fi
done

echo ""
echo "All shaders built successfully!"