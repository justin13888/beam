#!/usr/bin/env bash
# Deployment verification script for Beam
# This script checks if all required environment variables are set

set -e

echo "=== Beam Deployment Configuration Verification ==="
echo ""

# Check if .env file exists
if [ ! -f .env ]; then
    echo "❌ .env file not found!"
    echo "   Please copy .env.example to .env and configure it:"
    echo "   cp .env.example .env"
    exit 1
fi

echo "✅ .env file found"
echo ""

# Source the .env file
set -a
source .env
set +a

# Check critical variables
ERRORS=0

check_var() {
    local var_name=$1
    local var_value=${!var_name}
    local is_critical=${2:-false}
    
    if [ -z "$var_value" ]; then
        if [ "$is_critical" = "true" ]; then
            echo "❌ $var_name is not set (CRITICAL)"
            ERRORS=$((ERRORS + 1))
        else
            echo "⚠️  $var_name is not set (optional)"
        fi
    else
        echo "✅ $var_name = $var_value"
    fi
}

echo "Checking Database Configuration:"
check_var "POSTGRES_USER" true
check_var "POSTGRES_PASSWORD" true
check_var "POSTGRES_DB" true
check_var "DATABASE_URL" true
echo ""

echo "Checking Backend Configuration:"
check_var "BIND_ADDRESS" true
check_var "SERVER_URL" true
check_var "VIDEO_DIR" true
check_var "CACHE_DIR" true
check_var "ENABLE_METRICS"
check_var "RUST_LOG"
echo ""

echo "Checking Frontend Configuration:"
check_var "C_APP_TITLE"
check_var "C_STREAM_SERVER_URL" true
echo ""

echo "Checking Port Mappings:"
check_var "STREAM_HOST_PORT"
check_var "WEB_HOST_PORT"
check_var "POSTGRES_HOST_PORT"
echo ""

# Security warnings
echo "=== Security Checks ==="
if [ "$POSTGRES_PASSWORD" = "password" ]; then
    echo "⚠️  WARNING: Using default PostgreSQL password!"
    echo "   Please change POSTGRES_PASSWORD in production!"
fi

if [[ "$SERVER_URL" == *"localhost"* ]] || [[ "$C_STREAM_SERVER_URL" == *"localhost"* ]]; then
    echo "⚠️  INFO: Using localhost URLs (OK for development)"
fi

echo ""
echo "=== Summary ==="
if [ $ERRORS -eq 0 ]; then
    echo "✅ Configuration is valid! You can start the services with:"
    echo "   podman compose up -d"
    echo "   # or"
    echo "   docker compose up -d"
    exit 0
else
    echo "❌ Found $ERRORS critical configuration error(s)"
    echo "   Please fix the errors above before deploying"
    exit 1
fi
