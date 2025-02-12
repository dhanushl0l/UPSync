#!/bin/bash

set -e  

# Build the Rust binary
echo "Building upsync..."
cargo build --bin upsync --release 

# Install the binary
echo "Installing upsync to /usr/local/bin/"
sudo install -m 755 target/release/upsync /usr/local/bin/upsync

# Install the binary
echo "Running setup"
upsync setup

# Create systemd service file
SERVICE_FILE="/etc/systemd/system/upsync.service"

echo "Creating systemd service file at $SERVICE_FILE"
sudo bash -c "cat > $SERVICE_FILE" <<EOF
[Unit]
Description=Upsync Service
After=network.target

[Service]
ExecStart=/usr/local/bin/upsync server
Restart=always
User=$(whoami)
WorkingDirectory=$(pwd)

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd daemon, enable, and start the service
echo "Reloading systemd, enabling, and starting upsync service..."
sudo systemctl daemon-reload
sudo systemctl enable upsync
sudo systemctl start upsync

echo "Upsync service installed and running!"
