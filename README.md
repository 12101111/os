# Operating System

My "operating system" ! Written in Rust. Only support UEFI x86_64 now.

## Build tools

- [bootuefi](https://github.com/12101111/bootuefi)

- [OVMF](https://github.com/tianocore/tianocore.github.io/wiki/OVMF):

Install OVMF and set `bios = "<path to OVMF.fd>"` in `Cargo.toml`.

**Note**: you can download [Gerd Hoffmann's OVMF builds](https://www.kraxel.org/repos/jenkins/edk2/) (edk2.git-ovmf-x64*.noarch.rpm) and use `OVMF_CODE.fd`.

## Setting

You need provied a ttf font in `Cargo.toml`

```toml
[package.metadata.bootuefi.copy]
"/usr/share/fonts/sarasa/sarasa-mono-sc-regular.ttf" = "font.ttf"
```

## Run

```shell
cargo run --release
```

![os](./doc/os.png)

## TODO

The allocator is **just** enough to boot and draw on screen. It's inefficiency!
