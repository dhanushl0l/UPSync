#!/bin/bash

set -e 

# Build the Rust binary
echo "Building upsync-gui..."
cargo build --bin upsync-gui --release 

# Install the binary
echo "Installing upsync to /usr/local/bin/"
sudo install -m 755 target/release/upsync /usr/local/bin/upsync

# Path to the Polkit rules file
RULES_FILE="/etc/polkit-1/rules.d/50-suspend.rules"
# Polkit rule to add
POLKIT_RULE='polkit.addRule(function(action, subject) {
    if ((action.id == "org.freedesktop.login1.suspend" ||
         action.id == "org.freedesktop.login1.power-off" ||
         action.id == "org.freedesktop.login1.hibernate") &&
        subject.isInGroup("wheel")) { // Replace "wheel" with the correct group
        return polkit.Result.YES;
    }
});'
# Check if the rule already exists in the file
if sudo grep -qF "$POLKIT_RULE" "$RULES_FILE"; then
    echo "Polkit rule already exists. No changes made."
else
    echo "Adding Polkit rule to $RULES_FILE..."
    # Create the file if it does not exist
    sudo touch "$RULES_FILE"
    # Append the rule to the file if it's not already there
    echo "$POLKIT_RULE" | sudo tee -a "$RULES_FILE" > /dev/null
    # Restart Polkit service
    echo "Restarting Polkit service..."
    sudo systemctl restart polkit
    echo "Polkit rule added and Polkit service restarted successfully."
fi