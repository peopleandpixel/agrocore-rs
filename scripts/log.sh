#!/bin/bash
# log.sh - View AgroCore logs in a human-readable format
#
# Usage:
#   ./scripts/log.sh              # Show latest logs (all crates)
#   ./scripts/log.sh --follow     # Follow latest logs (like tail -f)
#   ./scripts/log.sh --errors     # Show only errors
#   ./scripts/log.sh --warnings   # Show warnings and errors
#   ./scripts/log.sh --crate api  # Show logs for specific crate
#   ./scripts/log.sh --tail 100   # Show last 100 lines
#   ./scripts/log.sh --help       # Show this help

set -e

LOG_DIR="${AGROCORE_LOG_DIR:-logs}"

show_help() {
    cat << 'EOF'
AgroCore Log Viewer

Usage: ./scripts/log.sh [OPTIONS]

Options:
  --follow, -f        Follow logs in real-time (like tail -f)
  --errors, -e        Show only ERROR level logs
  --warnings, -w      Show WARNING and ERROR level logs
  --crate <name>      Show logs for a specific crate (e.g., api, infrastructure)
  --tail <N>          Show last N lines (default: 50)
  --json              Output raw JSON (for piping to jq)
  --help, -h          Show this help message

Examples:
  ./scripts/log.sh                    # Last 50 lines, pretty format
  ./scripts/log.sh --errors           # Only errors
  ./scripts/log.sh --crate api -f     # Follow API logs
  ./scripts/log.sh --tail 200 -w      # Last 200 lines, warnings+

The logs are stored as JSONL (one JSON object per line).
This script parses them and displays in a readable format with colors.
EOF
}

# Colors
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color
BOLD='\033[1m'

FOLLOW=false
FILTER_LEVEL=""
CRATE_FILTER=""
TAIL_LINES=50
RAW_JSON=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --follow|-f)
            FOLLOW=true
            shift
            ;;
        --errors|-e)
            FILTER_LEVEL="ERROR"
            shift
            ;;
        --warnings|-w)
            FILTER_LEVEL="WARN"
            shift
            ;;
        --crate)
            CRATE_FILTER="$2"
            shift 2
            ;;
        --tail)
            TAIL_LINES="$2"
            shift 2
            ;;
        --json)
            RAW_JSON=true
            shift
            ;;
        --help|-h)
            show_help
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Check if jq is available
if ! command -v jq &> /dev/null; then
    echo "Warning: jq not found. Installing..."
    if command -v apt-get &> /dev/null; then
        sudo apt-get install -y jq 2>/dev/null
    elif command -v brew &> /dev/null; then
        brew install jq
    else
        echo "Error: jq is required but not installed. Please install it manually."
        exit 1
    fi
fi

# Find log files
get_log_files() {
    if [ -n "$CRATE_FILTER" ]; then
        # Look for specific crate logs
        find "$LOG_DIR" -name "*${CRATE_FILTER}*" -type f 2>/dev/null | sort
    else
        # Main log file + all crate logs
        find "$LOG_DIR" -name "*.json*" -o -name "*.jsonl*" -type f 2>/dev/null | sort
    fi
}

# Parse and format a single JSONL line
format_line() {
    local line="$1"
    
    # Skip empty lines
    [ -z "$line" ] && return
    
    # Try to parse as JSON
    local parsed
    parsed=$(echo "$line" | jq -r '
        def level_color:
            if . == "ERROR" then "\u001b[0;31m"
            elif . == "WARN" then "\u001b[1;33m"
            elif . == "INFO" then "\u001b[0;32m"
            elif . == "DEBUG" then "\u001b[0;36m"
            else "\u001b[0m"
            end;
        
        def level_emoji:
            if . == "ERROR" then "✗"
            elif . == "WARN" then "⚠"
            elif . == "INFO" then "ℹ"
            elif . == "DEBUG" then "🔍"
            else "•"
            end;
        
        "\(.timestamp | split("Z")[0] | split("T")[1]) \(.level | level_emoji) [\(.level | level_color)\(.level)\u001b[0m] \(.fields.message // "no message")"
    ' 2>/dev/null) || return
    
    echo "$parsed"
}

# Filter lines by level
filter_by_level() {
    local level="$1"
    
    case "$level" in
        ERROR)
            jq -r 'select(.level == "ERROR")' 2>/dev/null
            ;;
        WARN)
            jq -r 'select(.level == "ERROR" or .level == "WARN")' 2>/dev/null
            ;;
        *)
            cat
            ;;
    esac
}

# Main logic
log_files=$(get_log_files)

if [ -z "$log_files" ]; then
    echo "No log files found in $LOG_DIR"
    exit 1
fi

if [ "$RAW_JSON" = true ]; then
    # Raw JSON output for piping
    if [ "$FOLLOW" = true ]; then
        tail -f $log_files | filter_by_level "$FILTER_LEVEL"
    else
        cat $log_files | filter_by_level "$FILTER_LEVEL" | tail -n "$TAIL_LINES"
    fi
    exit 0
fi

if [ "$FOLLOW" = true ]; then
    # Follow mode
    echo -e "${BOLD}Following logs (Ctrl+C to stop)...${NC}"
    echo "---"
    tail -f $log_files | while IFS= read -r line; do
        [ -z "$line" ] && continue
        
        # Quick level filter without jq for performance
        if [ -n "$FILTER_LEVEL" ]; then
            case "$FILTER_LEVEL" in
                ERROR)
                    echo "$line" | jq -e 'select(.level == "ERROR")' > /dev/null 2>&1 || continue
                    ;;
                WARN)
                    echo "$line" | jq -e 'select(.level == "ERROR" or .level == "WARN")' > /dev/null 2>&1 || continue
                    ;;
            esac
        fi
        
        format_line "$line"
    done
else
    # One-shot mode
    echo -e "${BOLD}AgroCore Logs (last $TAIL_LINES entries)${NC}"
    echo "---"
    
    cat $log_files | tail -n "$TAIL_LINES" | while IFS= read -r line; do
        [ -z "$line" ] && continue
        
        # Quick level filter
        if [ -n "$FILTER_LEVEL" ]; then
            case "$FILTER_LEVEL" in
                ERROR)
                    echo "$line" | jq -e 'select(.level == "ERROR")' > /dev/null 2>&1 || continue
                    ;;
                WARN)
                    echo "$line" | jq -e 'select(.level == "ERROR" or .level == "WARN")' > /dev/null 2>&1 || continue
                    ;;
            esac
        fi
        
        format_line "$line"
    done
fi
