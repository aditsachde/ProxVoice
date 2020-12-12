# ProxVoice

Discord based proximity voice chat for Minecraft 1.12.2 

![Release](https://github.com/aditsachde/ProxVoice/workflows/Release/badge.svg)

## Usage

Download and place the ProxVoice jar in the Forge mod folder. Download and launch the desktop client that maches your OS. Make sure Discord is running before starting the client.

### Creating and Managing Lobbies

TODO

## Releases

Releases are automatically handled through Github Actions, however, no component currently updates automatically.

## Architecture

ProxVoice is split in two parts – the Minecraft mod and the desktop client – as the Discord Game SDK does not work easily with Java. It is completely client side.

### Mod

The mod simply takes player distances and publishes them on a ZeroMQ Pub/Sub socket, providing a light and nonblocking transport.

### Desktop Client

The UI is provided by a very simple electron app located in the client directory. The actual logic is provided a python webserver, located in the server directory. The server is bundled into the client on build.

## License

Everything is licensed under GPLv3