# Operating System

My "operating system" ! Written in Rust. Only support UEFI x86_64 now.

## Build tools

- [cargo-xbuild](https://github.com/rust-osdev/cargo-xbuild)

- [my uefi-run fork](https://github.com/12101111/uefi-run)

- [OVMF](https://github.com/tianocore/tianocore.github.io/wiki/OVMF):

Install OVMF using your distro's package manager and change `runner = "uefi-run"` to `runner = "uefi-run -b <OVMF PATH>"` in `.cargo/config`.

**Note**: if your distro's OVMF version is too old / does not provide these files,
  you can download [Gerd Hoffmann's builds](https://www.kraxel.org/repos/jenkins/edk2/edk2.git-ovmf-x64-0-20190621.1141.gc54c856218.noarch.rpm) and extract `OVMF_CODE.fd` to root directory and rename it to `OVMF.fd`.

## Run

```shell
cargo xrun
```

![os](./doc/os.jpg)
