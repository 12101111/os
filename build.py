#!/usr/bin/env python3

'Script used to build, run, and test the code on all supported platforms.'

import argparse
import filecmp
import json
import os
from pathlib import Path
import re
import shutil
import subprocess as sp
import sys

## Configurable settings
# Path to workspace directory (which contains the top-level `Cargo.toml`)
WORKSPACE_DIR = Path(__file__).resolve().parents[0]
PACKAGE = 'os'
# Try changing these with command line flags, where possible
SETTINGS = {
    # Direct Run From /EFI/Boot/Boot{Arch}.EFI
    'direct': True,
    # Print commands before running them.
    'verbose': False,
    # Run QEMU without showing GUI
    'headless': False,
    # Target to build for.
    'target': 'x86_64-unknown-uefi',
    # Configuration to build.
    'config': 'debug',
    # QEMU executable to use
    'qemu_binary': 'qemu-system-x86_64',
    # Path to directory containing `OVMF_{CODE/VARS}.fd`.
    # TODO: use installed OVMF, if available.
    'ovmf_dir': WORKSPACE_DIR / 'OVMF',
}

def build_dir():
    'Returns the directory where Cargo places the build artifacts'
    return WORKSPACE_DIR / 'target' / SETTINGS['target'] / SETTINGS['config']

def esp_dir():
    'Returns the directory where we will build the emulated UEFI system partition'
    return build_dir() / 'esp'

def boot_dir():
    return esp_dir() / 'EFI' / 'Boot'

def output_dir():
    'Return the directory where place the output EFI files in ESP'
    if SETTINGS['direct']:
        return boot_dir() / 'BootX64.efi'
    else:
        shutil.rmtree(boot_dir())
        return esp_dir() / (PACKAGE + '.efi')

def run_xtool(tool, *flags):
    'Runs cargo-x<tool> with certain arguments.'

    cmd = ['cargo', tool, '--target', SETTINGS['target'], *flags]

    if SETTINGS['verbose']:
        print(' '.join(cmd))

    sp.run(cmd).check_returncode()

def run_xbuild(*flags):
    'Runs cargo-xbuild with certain arguments.'
    run_xtool('xbuild', *flags)

def run_xclippy(*flags):
    'Runs cargo-xclippy with certain arguments.'
    run_xtool('xclippy', *flags)

def build(*test_flags):
    'Builds the tests and examples.'

    xbuild_args = [
        '--package', PACKAGE,
        *test_flags,
    ]

    if SETTINGS['config'] == 'release':
        xbuild_args.append('--release')

    run_xbuild(*xbuild_args)

    # Copy the built test runner file to the right directory for running tests.
    built_file = build_dir() / (PACKAGE + '.efi')

    boot_dir().mkdir(parents=True, exist_ok=True)
    
    shutil.copy2(built_file, output_dir())

def clippy():
    'Runs Clippy on all projects'

    run_xclippy('--all')

def doc():
    'Generates documentation for the library crates.'
    sp.run([
        'cargo', 'doc', '--no-deps',
        '--package', PACKAGE,
    ])

def run_qemu():
    'Runs the code in QEMU.'

    # Rebuild all the changes.
    build()

    ovmf_dir = SETTINGS['ovmf_dir']
    ovmf_code, ovmf_vars = ovmf_dir / 'OVMF_CODE.fd', ovmf_dir / 'OVMF_VARS.fd'

    if not ovmf_code.is_file():
        raise FileNotFoundError(f'OVMF_CODE.fd not found in the `{ovmf_dir}` directory')

    examples_dir = build_dir() / 'examples'

    #qemu_monitor_pipe = 'qemu-monitor'

    qemu_flags = [
        # Disable default devices.
        # QEMU by defaults enables a ton of devices which slow down boot.
        '-nodefaults',

        # Use a modern machine, with acceleration if possible.
        '-machine', 'q35,accel=kvm:tcg',

        # Allocate some memory.
        '-m', '128M',

        # Set up OVMF.
        '-drive', f'if=pflash,format=raw,file={ovmf_code},readonly=on',
        '-drive', f'if=pflash,format=raw,file={ovmf_vars},readonly=on',

        # Mount a local directory as a FAT partition.
        '-drive', f'format=raw,file=fat:rw:{esp_dir()}',

        # Mount the built examples directory.
        '-drive', f'format=raw,file=fat:rw:{examples_dir}',

        # Connect the serial port to the host. OVMF is kind enough to connect
        # the UEFI stdout and stdin to that port too.
        '-serial', 'stdio',

        # Map the QEMU exit signal to port f4
        '-device', 'isa-debug-exit,iobase=0xf4,iosize=0x04',

        # OVMF debug builds can output information to a serial `debugcon`.
        # Only enable when debugging UEFI boot:
        #'-debugcon', 'file:debug.log', '-global', 'isa-debugcon.iobase=0x402',
    ]

    # When running in headless mode we don't have video, but we can still have
    # QEMU emulate a display and take screenshots from it.
    qemu_flags.extend(['-vga', 'std'])
    if SETTINGS['headless']:
        # Do not attach a window to QEMU's display
        qemu_flags.extend(['-display', 'none'])

    cmd = [SETTINGS['qemu_binary']] + qemu_flags

    if SETTINGS['verbose']:
        print(' '.join(cmd))

    # This regex can be used to detect and strip ANSI escape codes when
    # analyzing the output of the test runner.
    ansi_escape = re.compile(r'(\x9B|\x1B\[)[0-?]*[ -/]*[@-~]')

    # Start QEMU
    qemu = sp.Popen(cmd, stdin=sp.PIPE, stdout=sp.PIPE, universal_newlines=True)
    try:
        # Iterate over stdout...
        for line in qemu.stdout:
            # Strip ending and trailing whitespace + ANSI escape codes
            # (This simplifies log analysis and keeps the terminal clean)
            stripped = ansi_escape.sub('', line.strip())

            # Skip lines which contain nothing else
            if not stripped:
                continue

            # Print out the processed QEMU output for logging & inspection
            print(stripped)

    finally:
        # Wait for QEMU to finish
        status = qemu.wait()

        # Throw an exception if QEMU failed
        if status != 0:
            raise sp.CalledProcessError(cmd=cmd, returncode=status)

def main():
    'Runs the user-requested actions.'

    # Clear any Rust flags which might affect the build.
    os.environ['RUSTFLAGS'] = ''

    usage = '%(prog)s verb [options]'
    desc = 'Build script for UEFI programs'

    parser = argparse.ArgumentParser(usage=usage, description=desc)

    parser.add_argument('verb', help='command to run', type=str,
                        choices=['build', 'run', 'doc', 'clippy'])

    parser.add_argument('--verbose', '-v', help='print commands before executing them',
                        action='store_true')

    parser.add_argument('--headless', help='run QEMU without a GUI',
                        action='store_true')

    parser.add_argument('--release', help='build in release mode',
                        action='store_true')

    opts = parser.parse_args()

    # Check if we need to enable verbose mode
    SETTINGS['verbose'] = opts.verbose
    SETTINGS['headless'] = opts.headless
    SETTINGS['config'] = 'release' if opts.release else 'debug'

    verb = opts.verb

    if verb == 'build':
        build()
    elif verb == 'clippy':
        clippy()
    elif verb == 'doc':
        doc()
    elif verb == 'run' or verb is None or opts.verb == '':
        # Run the program, by default.
        run_qemu()
    else:
        raise ValueError(f'Unknown verb {opts.verb}')

if __name__ == '__main__':
    try:
        main()
    except sp.CalledProcessError as cpe:
        print(f'Subprocess {cpe.cmd[0]} exited with error code {cpe.returncode}')
        sys.exit(1)
