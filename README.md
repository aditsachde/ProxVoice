# ProxVoice

Discord based proximity voice chat for Minecraft 1.12.2 

![Release](https://github.com/aditsachde/ProxVoice/workflows/Release/badge.svg)

## Usage

Download and place the ProxVoice jar in the Forge mod folder. Download and launch the desktop binary that maches your OS. Make sure Discord and Minecraft are running before starting the client. Additionally, make sure `proxvoice.config.yml` exists in the same directory as the binary.

### Creating and Managing Lobbies

TODO

## Releases

Releases are automatically handled through Github Actions, however, no component currently updates automatically.

## Architecture

ProxVoice is split in two parts – the Minecraft mod and the desktop client.

### Mod

The mod simply takes player distances and publishes them on a ZeroMQ Pub/Sub socket, providing a light and nonblocking transport.

### Desktop Client

Currently, this is a rust binary that does not provide a GUI. Config is specified using `proxvoice.config.yml`

## License

Everything is licensed under GPLv3