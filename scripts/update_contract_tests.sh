#!/bin/bash
# API Contract Updater Script v2
#
# Scans crates/admin-ui/src/api.rs for fetch_* function definitions
# and verifies they appear in the mock_test_framework.rs scenarios.
#
# Run: bash scripts/update_contract_tests.sh

echo "🔍 Scanning for fetch_* functions in admin-ui/src/api.rs..."

# Only check fetch_* functions (these should all be in mock scenarios)
# The regex captures the full function name (fetch_xxx), then we compare against
# both "fetch_xxx" and "xxx" in the test file
ALL_FUNCS=$(grep -oP 'pub async fn fetch_\K\w+' crates/admin-ui/src/api.rs | sort -u)
# Rebuild full names with fetch_ prefix
CORE_FETCH_FUNCS=$(grep -oP 'pub async fn \Kfetch_\w+' crates/admin-ui/src/api.rs | sort -u)

echo "📋 Core fetch functions to check in mock scenarios:"
echo "$CORE_FETCH_FUNCS"
echo ""

# Count functions missing from mock scenarios
MISSING=0
MISSING_LIST=()
for fn_name in $CORE_FETCH_FUNCS; do
    # Check if the full "fetch_xxx" name appears in the test file
    if ! grep -q "\"${fn_name}\"" crates/admin-ui/src/tests/mock_test_framework.rs; then
        MISSING=$((MISSING + 1))
        MISSING_LIST+=("$fn_name")
    fi
done

TOTAL=$(echo "$CORE_FETCH_FUNCS" | wc -l | tr -d ' ')
COVERED=$((TOTAL - MISSING))

echo "📊 Coverage: ${COVERED}/${TOTAL} fetch functions covered by mock scenarios"

if [ $MISSING -gt 0 ]; then
    echo ""
    echo "⚠️  ${MISSING} functions missing from mock scenarios:"
    for fn_name in "${MISSING_LIST[@]}"; do
        echo "   - ${fn_name}"
    done
    echo ""
    echo "To add them, update scenarios::all_500(), network_failure(),"
    echo "auth_expired(), json_malformed() and test_all_api_functions_covered_by_scenarios()"
    echo ""
    echo "Note: Only fetch_* functions need mock coverage."
    echo "Mutation functions (create_/update_/delete_) are validated differently."
    exit 1
fi

echo ""
echo "✅ All ${TOTAL} fetch functions are covered by mock scenarios!"
