#!/bin/bash
# API Contract Updater Script
#
# Scans crates/admin-ui/src/api.rs for fetch_* function definitions
# and verifies they appear in the mock_test_framework.rs scenarios.
# Only checks fetch_* and key mutation functions (create_, delete_).
#
# Run: bash scripts/update_contract_tests.sh

echo "🔍 Scanning for fetch_* and mutation functions in admin-ui/src/api.rs..."

# Only check the CORE fetch_ functions that should be in all_500/network/auth/json scenarios
CORE_FETCH_FUNCS=$(grep -oP 'pub async fn fetch_\K\w+' crates/admin-ui/src/api.rs | sort -u)

echo "📋 Core fetch functions to check in mock scenarios:"
echo "$CORE_FETCH_FUNCS"
echo ""

# Check if each fetch function appears in mock scenarios
MISSING=()
for fn_name in $CORE_FETCH_FUNCS; do
    if ! grep -q "\"$fn_name\"" crates/admin-ui/src/tests/mock_test_framework.rs; then
        MISSING+=("$fn_name")
        echo "⚠️  fetch_$fn_name NOT found in mock_test_framework.rs"
    fi
done

# Also check key mutation functions in test_crud_operations_can_error
MUTATION_FUNCS=$(grep -oP 'pub async fn (create_|update_|delete_|clock_|stock_|transfer_|adjust_)\K\w+' crates/admin-ui/src/api.rs | sed 's/ //g' | sort -u)
MUTATION_MISSING=()
for fn_name in $MUTATION_FUNCS; do
    if ! grep -q "\"$fn_name\"" crates/admin-ui/src/tests/mock_test_framework.rs 2>/dev/null; then
        # Only check mutations that should be in test_crud_operations_can_error
        if echo "$fn_name" | grep -qE "^(create_|delete_|update_|clock_out|stock_in|stock_out|transfer_inventory|adjust_inventory)$"; then
            MUTATION_MISSING+=("$fn_name")
            echo "⚠️  Mutation $fn_name NOT found in test_crud_operations_can_error!"
        fi
    fi
done

if [ ${#MISSING[@]} -gt 0 ]; then
    echo ""
    echo "❌ ${#MISSING[@]} fetch functions missing from mock scenarios!"
    echo "Please add them to:"
    echo "  - scenarios::all_500()"
    echo "  - scenarios::network_failure()"
    echo "  - scenarios::auth_expired()"
    echo "  - scenarios::json_malformed()"
    echo "  - test_all_api_functions_covered_by_scenarios() list"
    exit 1
fi

if [ ${#MUTATION_MISSING[@]} -gt 0 ]; then
    echo ""
    echo "⚠️  ${#MUTATION_MISSING[@]} mutation functions missing from test_crud_operations_can_error!"
fi

echo ""
echo "✅ All ${#CORE_FETCH_FUNCS[@]} core fetch functions are covered by mock scenarios!"
