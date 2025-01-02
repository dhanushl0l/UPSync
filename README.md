# Smart-UPS

Smart-UPS is a binary application designed to convert a non-smart UPS into a functional smart UPS by leveraging the charging state of a connected laptop. The application monitors the laptop's power state (charging or discharging) to infer the presence of mains power. Based on this, it can send commands to manage a connected PC, such as powering it off or waking it up using Wake-on-LAN.

This project is under development and is designed for niche scenarios. It's specifically useful in setups where:

    A laptop is used as a server.
    A desktop PC is connected to a UPS.
    Both devices are on the same local network.

## Features

    Power Monitoring: Tracks the laptop's charging state to detect power outages.
    Power-off Command: Sends a power-off command to the connected PC during power outages.
    Wake-on-LAN: Sends a Wake-on-LAN (WoL) packet to the PC when power is restored.
    Configurable: Settings can be customized to fit specific setups and requirements.

## Limitations

This tool is not intended for general-purpose use and is only applicable in specific scenarios:

    Requires a laptop with power state monitoring capabilities.
    Both devices must be connected to the same network.
    Designed for learning and experimental purposes.

Why Rust?

This application is written in Rust, primarily as a learning project to explore and practice Rust programming as a beginner.
