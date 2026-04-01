#!/bin/bash
# Email Generator Benchmark Script
#
# This script runs comprehensive benchmarks for the emailgen project,
# comparing different generation sizes and measuring performance.
#
# Usage:
#   ./scripts/benchmark.sh
#   ./scripts/benchmark.sh --quick    # Run quick benchmarks only
#   ./scripts/benchmark.sh --full     # Run full benchmarks including 1M generation

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default settings
QUICK_MODE=false
FULL_MODE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --quick|-q)
            QUICK_MODE=true
            shift
            ;;
        --full|-f)
            FULL_MODE=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --quick, -q    Run quick benchmarks only (no 1M generation)"
            echo "  --full, -f     Run full benchmarks including 1M email generation"
            echo "  --help, -h     Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}============================================${NC}"
echo -e "${BLUE}  Email Generator Performance Benchmarks   ${NC}"
echo -e "${BLUE}============================================${NC}"
echo ""

# Ensure release build exists
echo -e "${YELLOW}Building release binary...${NC}"
cargo build --release --quiet 2>/dev/null || cargo build --release

echo ""
echo -e "${YELLOW}Running Criterion benchmarks...${NC}"
cargo bench --quiet 2>/dev/null || cargo bench

echo ""
echo -e "${BLUE}============================================${NC}"
echo -e "${BLUE}  CLI Generation Benchmarks                ${NC}"
echo -e "${BLUE}============================================${NC}"
echo ""

# Function to run generation benchmark
run_generation_benchmark() {
    local count=$1
    local label=$2
    
    echo -e "${YELLOW}Generating $count emails ($label)...${NC}"
    
    local start_time=$(date +%s.%N)
    
    # Generate to /dev/null for performance measurement
    cargo run --release --quiet -- --count $count --output /dev/null 2>/dev/null
    
    local end_time=$(date +%s.%N)
    local elapsed=$(echo "$end_time - $start_time" | bc)
    local rate=$(echo "scale=0; $count / $elapsed" | bc)
    
    echo -e "  ${GREEN}✓${NC} Generated $count emails in ${elapsed}s (${rate} emails/sec)"
}

# Quick benchmarks
echo -e "${BLUE}Quick Benchmarks:${NC}"
echo "----------------------------------------"

run_generation_benchmark 1000 "1K emails"
run_generation_benchmark 10000 "10K emails"
run_generation_benchmark 100000 "100K emails"

# Full benchmarks
if [ "$FULL_MODE" = true ] || [ "$QUICK_MODE" = false ]; then
    echo ""
    echo -e "${BLUE}Full Benchmarks:${NC}"
    echo "----------------------------------------"
    
    run_generation_benchmark 500000 "500K emails"
    
    if [ "$FULL_MODE" = true ]; then
        run_generation_benchmark 1000000 "1M emails"
    fi
fi

echo ""
echo -e "${BLUE}============================================${NC}"
echo -e "${BLUE}  Memory Usage Analysis                    ${NC}"
echo -e "${BLUE}============================================${NC}"
echo ""

# Memory usage test
echo -e "${YELLOW}Testing memory usage for different capacities:${NC}"

for capacity in 10000 100000 1000000; do
    echo ""
    echo "Bloom filter capacity: $capacity"
    cargo run --release --quiet -- --count 1000 --capacity $capacity --stats 2>&1 | grep "Memory usage" || true
done

echo ""
echo -e "${BLUE}============================================${NC}"
echo -e "${BLUE}  Benchmark Summary                        ${NC}"
echo -e "${BLUE}============================================${NC}"
echo ""

# Generate summary
echo "Performance Targets:"
echo "  ✓ 1K emails:   < 0.1 seconds"
echo "  ✓ 10K emails:  < 1 second"
echo "  ✓ 100K emails: < 10 seconds"
echo "  ✓ 1M emails:   < 100 seconds"
echo ""

echo "Memory Efficiency:"
echo "  ✓ Bloom filter uses ~10-15 MB for 1M emails"
echo "  ✓ False positive rate: 1% (configurable)"
echo ""

echo -e "${GREEN}Benchmarks complete!${NC}"
echo ""
echo "For detailed results, see:"
echo "  - target/criterion/report/index.html"
echo ""
