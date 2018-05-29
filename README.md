# Minecraft Modpack Update Tool
This tool is built to enable Modpack creators to automatically update their Minecraft Modpacks in a platform-independent manner, based upon the TOML data format.

## Configuration

The configuration file (`mpupd.toml`) used by this tool must contain a single TOML-formatted table, called `channels`. Each key in the table must be the name of an update channel, with the corresponding value being the URL at which that channel's TOML file can be found (See the next section for more information). **By default, the configuration file does not contain valid data. This is the responsibility of the modpack author.**

Example: 

```toml
[channels]
stable = "https://www.some-website.com/channels/stable.toml"
beta = "https://www.some-website.com/channels/beta.toml"
```

### Channel file

Each update channel is required to have a single TOML file made available at the location specified in the configuration file. The channel TOML file is simply an array of strings called `updates`, each being the URL of a particular released update, also specified in TOML (see next section for more information). **These URLs must never change.** When a new update is released, simply edit this file and add an new entry.

Example:

`https://www.some-website.com/channels/stable.toml`

```toml
updates = [
    "https://www.some-website.com/channels/stable/1.0.toml",
    "https://www.some-website.com/channels/stable/1.1.toml",
]
```

### Update File

Each update for each channel must be specified in its own TOML-formatted file, available at the URL specified by the channel. This file contains information on individual files that must be added or removed in order to update the modpack. All directory locations are relative to the location of the executable, and it is recommended to place the executable within the `.minecraft` directory. **If a file needs to be replaced, register it as an addition ONLY. Additions automatically overwrite existing files.**

Example

`https://www.some-website.com/channels/stable/1.0/toml`

```toml
# An example mod addition
[[addition]] # Uses TOML's array-of-tables syntax. All additions must begin with this line
loc = "mods/whatever.jar" # Final location for the downloaded file.
url = "https://minecraft.curseforge.com/some-random-mod/download" # Location from which the new file can be downloaded.
sha256sum = "01ba4719c80b6fe911b091a7c05124b64eeece964e09c058ef8f9805daca546b" # SHA256 Checksum of the file at the url above. Used to verify the integrity of the downloaded file.

# An example config file addition
[[addition]]
loc = "config/whatever.cfg"
url = "https://www.some-website.com/wherever/the/file/is.txt"
sha256sum = "87428fc522803d31065e7bce3cf03fe475096631e5e07bbd7a0fde60c4cf25c7"

# An example mod deletion
[[deletion]] # All deletions must begin with this line
loc = "mods/some-other-mod.jar" # Local location of the file to be deleted.
sha256sum = "0263829989b6fd954f72baaf2fc64bc2e2f01d692d4de72986ea808f6e99813f" # SHA256 Checksum of the local file. The file will **ONLY** be deleted if the checksum is valid.
```

## Build Instructions

Dependencies:

* [Rust](https://rustup.rs) >= 1.26

Instructions:

```bash
    git clone https://github.com/zack-emmert/mpupd.git
    cd mpupd
    cargo build --release
    cd target/release
    cp mpupd /path/to/.minecraft
```
Additionally, copy the template configuration file in the project root to the same location, and adjust the listed channels according to the Modpack.

Alternatively, download the binary for your platform on the releases page.

## Embedding the executable into a Modpack

It is recommended that the executable be embedded into the same download as the Modpack itself, and be set to run on startup of the Minecraft, prior to the game itself. While the exact implementation details are left to the Modpack creator, the MultiMC launcher offers a "Pre-Launch Command" option suitable for this purpose. Note that it **must** be possible to specify command-line parameters from the method chosen, as the `-c` parameter is used to define the channel followed by the program.

Additionally, separate Modpack downloads must be provided for Windows, macOS, and Linux users, due to the nature of the language in which this tool is written.

### Final Notes

The Mac binaries available on the releases page are not signed, nor will they probably ever be.