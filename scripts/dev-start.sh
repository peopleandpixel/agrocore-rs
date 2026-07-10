#!/bin/bash
# Development Setup - agrocore-rs

set -e

echo "=== AGROCORE RS - LOCAL DEVELOPMENT ==="

# Start Docker Services
echo "Starting MongoDB, NATS, Grafana, API..."
docker-compose up -d mongodb nats prometheus grafana api

# Wait for API
echo "Waiting for API to be ready..."
sleep 5

# Check API Health
curl -s http://localhost:3000/api/v1/health | jq . || echo "API not responding yet"

echo "✅ Services started. Access:"
echo "  API: http://localhost:3000"
echo "  Grafana: http://localhost:3001 (admin/admin)"
echo "  Prometheus: http://localhost:9090"