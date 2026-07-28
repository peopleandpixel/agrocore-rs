#!/bin/bash
# SIGPAC Import Runner Script
# Usage: ./run_import.sh [region] [year] [options]

set -e

# Default values
REGION="${1:-all}"
YEAR="${2:-2023}"
DOWNLOAD_DIR="./data/sigpac"
BATCH_SIZE=10000
LIMIT=""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}╔══════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║     SIGPAC Reference Data Import                          ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════╝${NC}"

# Check DATABASE_URL
if [ -z "$DATABASE_URL" ]; then
    echo -e "${YELLOW}DATABASE_URL not set, using default${NC}"
    export DATABASE_URL="postgresql://agrocore:agrocore-secret@localhost:5432/agrocore"
fi

# Check Python venv
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VENV_DIR="$SCRIPT_DIR/venv"

if [ ! -d "$VENV_DIR" ]; then
    echo -e "${YELLOW}Creating virtual environment...${NC}"
    python3 -m venv "$VENV_DIR"
    source "$VENV_DIR/bin/activate"
    pip install --upgrade pip
    pip install -r "$SCRIPT_DIR/requirements.txt"
else
    source "$VENV_DIR/bin/activate"
fi

# Create download directory
mkdir -p "$DOWNLOAD_DIR"

# Parse additional arguments
shift 2 2>/dev/null || true
while [[ $# -gt 0 ]]; do
    case $1 in
        --limit)
            LIMIT="$2"
            shift 2
            ;;
        --batch-size)
            BATCH_SIZE="$2"
            shift 2
            ;;
        --download-dir)
            DOWNLOAD_DIR="$2"
            shift 2
            ;;
        --skip-download)
            SKIP_DOWNLOAD="--skip-download"
            shift
            ;;
        --dry-run)
            DRY_RUN="--dry-run"
            shift
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Build command
CMD="python $SCRIPT_DIR/import_sigpac.py"
CMD+=" --region $REGION"
CMD+=" --year $YEAR"
CMD+=" --batch-size $BATCH_SIZE"
CMD+=" --download-dir $DOWNLOAD_DIR"

if [ -n "$LIMIT" ]; then
    CMD+=" --limit $LIMIT"
fi

if [ -n "$SKIP_DOWNLOAD" ]; then
    CMD+=" $SKIP_DOWNLOAD"
fi

if [ -n "$DRY_RUN" ]; then
    CMD+=" $DRY_RUN"
fi

echo -e "${GREEN}Running:${NC} $CMD"
echo ""

eval $CMD

echo ""
echo -e "${GREEN}Import complete!${NC}"