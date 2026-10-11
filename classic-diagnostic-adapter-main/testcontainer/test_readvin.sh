#!/bin/bash
set -e

COMPONENT_ID="${COMPONENT_ID:-e400}"

echo "1. Getting Token..."
TOKEN=$(curl -s -X POST "http://localhost:36095/vehicle/v15/authorize" \
  -H "Content-Type: application/json" \
  -d '{"client_id": "test_client", "client_secret": "test_secret"}' | grep -o '"access_token":"[^"]*' | cut -d'"' -f4)
echo "Token: $TOKEN"

echo -e "\n2. Reading VIN via the data API..."
VIN_RESPONSE=$(curl -s -X GET "http://localhost:36095/vehicle/v15/components/${COMPONENT_ID}/data/ReadVIN" \
  -H "Authorization: Bearer $TOKEN")
echo "$VIN_RESPONSE"
