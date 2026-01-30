#!/bin/bash
# Generate code coverage report and show uncovered lines
#
# Usage:
#   ./scripts/coverage-report.sh           # Show summary
#   ./scripts/coverage-report.sh --html    # Generate HTML report and open in browser
#   ./scripts/coverage-report.sh --missing # Show uncovered lines

set -e

MODE="${1:-summary}"

case "$MODE" in
    --html)
        echo "Generating HTML coverage report..."
        cargo llvm-cov --html --open
        ;;
    --missing)
        echo "Finding uncovered lines..."
        cargo llvm-cov --json --output-path target/llvm-cov.json >/dev/null 2>&1
        
        # Parse JSON and show files with uncovered lines
        jq -r '
            .data[].files[] | 
            select(.summary.regions.count > .summary.regions.covered) |
            "\(.filename):\n  Coverage: \(.summary.lines.percent)%\n  Uncovered lines: \(.summary.lines.count - .summary.lines.covered)/\(.summary.lines.count)\n"
        ' target/llvm-cov.json
        
        echo ""
        echo "For detailed line-by-line coverage, run: cargo llvm-cov --text"
        ;;
    summary|*)
        echo "Running tests with coverage..."
        cargo llvm-cov --summary-only
        ;;
esac
