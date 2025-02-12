# UPSync

UPSync is a binary application designed to convert a non-smart UPS into a functional smart UPS by leveraging the charging state of a connected laptop. The application monitors the specific device's power state (charging or discharging) to infer the presence of mains power. Based on this, it can send commands to manage a connected PC Via SSH, no additional service is required to run on the client. It starts the app installed on the client, and based on the user's response, it either sends the PC to sleep, hibernates it, or performs a custom action. The app also wakes the PC up using Wake-on-LAN when the power is restored.

### This project is under development and is designed for niche scenarios. It's specifically useful in setups where:

    Power Monitoring: Tracks the laptop's charging state to detect power outages.
    A laptop connected to a wall outlet and capable of running on battery power when the main power is out is used as a server.
    A desktop PC is connected to a UPS.
    Both devices are on the same local network.

Note: Instead of a laptop, a Raspberry Pi or any computer that can monitor home power could be used. This is not part of the current development plan but can be implemented in the future.

## Features

    Power Monitoring: Tracks the laptop's charging state to detect power outages.
    Power-off Command: Sends a power-off command to the connected PC during power outages.
    Wake-on-LAN: Sends a Wake-on-LAN (WoL) packet to the PC when power is restored.
    Configurable: Settings can be customized to fit specific setups and requirements.

## Limitations

This tool is not intended for general-purpose use and is only applicable in specific scenarios:

    Requires a laptop with a battery that can operate when the power is out.
    Both devices must be connected to the same network.
    Designed for learning and experimental purposes.

# 📌 Installation Guide

## Prerequisites

Before proceeding, ensure you have `git` installed. If not, install it using the following command:

### Arch Linux:

```bash
sudo pacman -S git
```

### Debian/Ubuntu:

```bash
sudo apt update && sudo apt install git -y
```

### Install Rust Build Tools

The project requires Rust and its build tools. Install them using:

```bash
sudo pacman -S rust
```

### Debian/Ubuntu:

```bash
sudo apt update && sudo apt install cargo rustc -y
```

You are free to remove the build tools after the installation is complete.

### Clone the Repository

Now, clone the repository and navigate into it:

```bash
git clone https://github.com/dhanushl0l/UPSync.git
cd UPSync
```

### Installation Steps

For Server Installation:

```bash
./install-server.sh
```

```bash
./install-client.sh
```

## Done!

Your setup is complete. If needed, you can further configure the application by editing the `config.config` file and tweaking environment variables. For more details, see the [`documentation`](not-implemented).

### Why Rust?

This application is written in Rust, primarily as a learning project to explore and practice Rust programming as a beginner.
