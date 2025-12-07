# rpgm-archive-decrypter

RPGM Archive Decrypter is a [RPG Maker Decrypter](https://github.com/uuksu/rpgmakerdecrypter) [rewrite in Rust](https://github.com/savannstm/rpgm-archive-decrypter-lib) (**_BLAZINGLY FAST_** :fire:).

It can be used to extract encrypted archives of RPG Maker XP/VX/VXAce game engines.

It is faster and lighter than RPG Maker Decrypter, and also has **NO** requirements to run, except a working PC.

_And also features much more cleaner code!_

## Installation

Get required binaries in **Releases** section.

## Usage

Call `rpgmad -h` for help.

### Decryption

For example, to extract archive to same same directory where it exists:
`rpgmad decrypt C:/Game/Archive.rgssad`.

You can omit the file name, program will find it automatically: `rpgmad C:/Game`.

Or just `rpgmad decrypt` if you're already in the game directory.

You can recongnize archives by their extensions: `rgssad`, `rgss2a`, `rgss3a`.

### Encryption

`rpgmad` encrypts entries from `Data` and `Graphics` directories (default behavior, directories can be altered using `--encrypt-dirs` and `--additional-encrypt-dirs` arguments) back to the archives.

Encryption requires an `--engine` argument for proper encryption to the correct archive.

For example, to encrypt Data/Graphics directories to a `Game.rgss3a` archive: `rpgmad encrypt C:/Game`.

Or just `rpgmad encrypt` if you're already in the game directory.

## GUI

Our [rpgmdec](https://github.com/rpg-maker-translation-tools/rpgmdec) GUI provides the same functionality as `rpgmad`.

## Development

### Building

Requirements: `rustup` with installed Rust toolchain.

Clone the repository and compile with `cargo b -r`.

### Tests

I'm not really skilled in tests, but the validity of output files is tested the following ways:

-   Of images, using image viewers.
-   Of rx/rvdata files, using [rvpacker-txt-rs](https://github.com/savannstm/rvpacker-txt-rs).

As long as these tests succesful, there shouldn't be any bugs.

## License

Project is licensed under WTFPL.
