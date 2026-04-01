#!/bin/bash
# Quick Email Generation Script
#
# Usage:
#   ./scripts/generate.sh [count] [output_file]
#
# Examples:
#   ./scripts/generate.sh              # Generate 100 emails to stdout
#   ./scripts/generate.sh 1000         # Generate 1K emails to stdout
#   ./scripts/generate.sh 10000 emails.txt  # Generate 10K to file

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_DIR"

# Default values
COUNT=${1:-100}
OUTPUT=${2:-""}

# Build if needed
if [ ! -f "target/release/emailgen" ]; then
    echo "Building emailgen..."
    cargo build --release --quiet
fi

# Generate
if [ -n "$OUTPUT" ]; then
    echo "Generating $COUNT emails to $OUTPUT..."
    ./target/release/emailgen --count $COUNT --output "$OUTPUT" --progress
    echo "Done! Generated $COUNT emails."
    echo "File size: $(ls -lh "$OUTPUT" | awk '{print $5}')"
else
    ./target/release/emailgen --count $COUNT --quiet
fi
