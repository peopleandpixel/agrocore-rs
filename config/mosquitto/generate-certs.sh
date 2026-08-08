#!/bin/bash
# Generate TLS certificates for Mosquitto MQTT broker
# Run this script to create self-signed certificates for development

set -e

CERT_DIR="/home/jens/RustroverProjects/agrocore-rs/config/mosquitto/certs"
mkdir -p "$CERT_DIR"

cd "$CERT_DIR"

echo "Generating CA certificate..."
openssl req -x509 -newkey rsa:4096 -sha256 -days 3650 -nodes \
  -keyout ca.key -out ca.crt \
  -subj "/CN=agrocore-ca/O=agrocore-rs"

echo "Generating server certificate..."
openssl req -newkey rsa:4096 -sha256 -nodes \
  -keyout server.key -out server.csr \
  -subj "/CN=mqtt.agrocore.local/O=agrocore-rs"

openssl x509 -req -sha256 -days 825 \
  -in server.csr -CA ca.crt -CAkey ca.key -CAcreateserial \
  -out server.crt

echo "Generating client certificate (for Home Assistant)..."
openssl req -newkey rsa:4096 -sha256 -nodes \
  -keyout client.key -out client.csr \
  -subj "/CN=homeassistant/O=agrocore-rs"

openssl x509 -req -sha256 -days 825 \
  -in client.csr -CA ca.crt -CAkey ca.key -CAcreateserial \
  -out client.crt

echo "Creating PKCS12 for Home Assistant..."
openssl pkcs12 -export \
  -in client.crt -inkey client.key \
  -certfile ca.crt \
  -out homeassistant.p12 \
  -passout pass:changeme

echo "Setting permissions..."
chmod 644 ca.crt server.crt client.crt homeassistant.p12
chmod 600 ca.key server.key client.key

echo "Certificates generated in $CERT_DIR:"
ls -la "$CERT_DIR"

echo ""
echo "To enable TLS in mosquitto.conf, uncomment the TLS listener section and update paths."
echo "For Home Assistant, import homeassistant.p12 with password 'changeme'"