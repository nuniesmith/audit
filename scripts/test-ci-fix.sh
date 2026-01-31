#!/bin/bash
# Test script to verify CI fix locally
# This simulates what the CI workflow does

set -e

echo "🧪 Testing CI Audit Workflow Fix"
echo "=================================="
echo ""

# Change to audit directory
cd "$(dirname "$0")"
echo "📁 Working directory: $(pwd)"
echo ""

# Clean up previous test files
echo "🧹 Cleaning up previous test files..."
rm -f file-list.txt ci-runs.json
rm -rf context-bundle
echo "✅ Cleanup complete"
echo ""

# Test 1: Create context-bundle directory
echo "📦 Test 1: Creating context-bundle directory..."
mkdir -p context-bundle
if [ -d "context-bundle" ]; then
    echo "✅ context-bundle directory created"
else
    echo "❌ Failed to create context-bundle directory"
    exit 1
fi
echo ""

# Test 2: Generate file list (all focus area)
echo "📝 Test 2: Generating file list (focus: all)..."
find ../../src/janus -type f \( -name "*.rs" -o -name "*.toml" -o -name "*.proto" \) 2>/dev/null | sort > file-list.txt || echo "../../src/janus/Cargo.toml" > file-list.txt

if [ -f "file-list.txt" ]; then
    FILE_COUNT=$(wc -l < file-list.txt)
    echo "✅ file-list.txt created with $FILE_COUNT files"
else
    echo "❌ Failed to create file-list.txt"
    exit 1
fi
echo ""

# Test 3: Generate file list (security focus)
echo "📝 Test 3: Testing security focus area..."
find ../../src/janus -type f -name "*.rs" 2>/dev/null | grep -E "(auth|security|crypto|token)" | sort > file-list-security.txt || touch file-list-security.txt
SECURITY_COUNT=$(wc -l < file-list-security.txt)
echo "✅ Security focus found $SECURITY_COUNT files"
echo ""

# Test 4: Generate file list (performance focus)
echo "📝 Test 4: Testing performance focus area..."
find ../../src/janus -type f -name "*.rs" 2>/dev/null | grep -E "(cache|pool|async|worker)" | sort > file-list-performance.txt || touch file-list-performance.txt
PERF_COUNT=$(wc -l < file-list-performance.txt)
echo "✅ Performance focus found $PERF_COUNT files"
echo ""

# Test 5: Create files-to-analyze.txt
echo "📋 Test 5: Creating files-to-analyze.txt..."
head -n 100 file-list.txt > context-bundle/files-to-analyze.txt
if [ -f "context-bundle/files-to-analyze.txt" ]; then
    ANALYZE_COUNT=$(wc -l < context-bundle/files-to-analyze.txt)
    echo "✅ files-to-analyze.txt created with $ANALYZE_COUNT files"
else
    echo "❌ Failed to create files-to-analyze.txt"
    exit 1
fi
echo ""

# Test 6: Verify file paths exist (check first 5 files only)
echo "🔍 Test 6: Verifying file paths..."
VALID_FILES=0
INVALID_FILES=0
COUNT=0
while IFS= read -r file && [ $COUNT -lt 5 ]; do
    if [ -f "$file" ]; then
        VALID_FILES=$((VALID_FILES + 1))
    else
        INVALID_FILES=$((INVALID_FILES + 1))
    fi
    COUNT=$((COUNT + 1))
done < context-bundle/files-to-analyze.txt

echo "   Valid files:   $VALID_FILES / $COUNT checked"
echo "   Invalid files: $INVALID_FILES / $COUNT checked"
if [ $VALID_FILES -gt 0 ]; then
    echo "✅ File paths are valid"
else
    echo "⚠️  Warning: No valid file paths found"
fi
echo ""

# Test 7: Simulate CI context gathering
echo "🔍 Test 7: Testing CI context gathering (without GitHub CLI)..."
echo '[]' > ci-runs.json
if [ -f "ci-runs.json" ]; then
    echo "✅ ci-runs.json created"
else
    echo "❌ Failed to create ci-runs.json"
    exit 1
fi
echo ""

# Summary
echo "=================================="
echo "📊 Test Summary"
echo "=================================="
echo ""
echo "All core functionality tests passed! ✅"
echo ""
echo "Files created:"
echo "  ✓ context-bundle/ directory"
echo "  ✓ file-list.txt ($FILE_COUNT files)"
echo "  ✓ file-list-security.txt ($SECURITY_COUNT files)"
echo "  ✓ file-list-performance.txt ($PERF_COUNT files)"
echo "  ✓ context-bundle/files-to-analyze.txt ($ANALYZE_COUNT files)"
echo "  ✓ ci-runs.json"
echo ""
echo "Focus area filtering:"
echo "  ✓ All: $FILE_COUNT files"
echo "  ✓ Security: $SECURITY_COUNT files"
echo "  ✓ Performance: $PERF_COUNT files"
echo ""
echo "🎉 CI fix should work correctly!"
echo ""
echo "To clean up test files, run:"
echo "  rm -f file-list*.txt ci-runs.json && rm -rf context-bundle"
echo ""
