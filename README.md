# FbwfCfgGen

[简体中文](README.zh.md) | English

`FbwfCfgGen` is a Rust command-line tool that generates the binary `fbwf.cfg` configuration used by the Windows Embedded File-Based Write Filter (FBWF). The tool creates a new 4096-byte configuration from a capacity value.

## Usage

```text
FbwfCfgGen <CAPACITY_MB> <OUTPUT_PATH> [--force]
```

Examples:

```text
FbwfCfgGen 4096 D:\WinPE\Windows\fbwf.cfg
FbwfCfgGen 8192 D:\WinPE\Windows\custom.cfg --force
```

Arguments and options:

- `CAPACITY_MB`: Capacity in megabytes. It must be between `128` and `131072`, inclusive, and must be a multiple of 16.
- `OUTPUT_PATH`: Destination file path. The parent directory must already exist; the file name is not restricted.
- `--force`, `-f`: Allow an existing destination file to be overwritten. No `.bak` file is created.

The command exits with `0` on success and a non-zero code when validation or file writing fails. All command-line messages are printed in English.

## Build

Build and test with Cargo:

```text
cargo build --release
cargo test
```

The release executable is written to `target/release/FbwfCfgGen.exe` on Windows.

## Generated file format

Every generated file is exactly `0x1000` bytes (4096 bytes). The buffer is initialized to zero, then the following fields are written:

| Offset |     Size | Value                          | Purpose                                 |
| -----: | -------: | ------------------------------ | --------------------------------------- |
| `0x00` |        1 | `0x24 - byte[0x1C]` modulo 256 | Check byte A                            |
| `0x01` |        2 | `F9 23`                        | Fixed signature, little-endian `0x23F9` |
| `0x03` |        1 | `0xF8 - byte[0x1B]` modulo 256 | Check byte B                            |
| `0x04` |        1 | `02`                           | Template constant                       |
| `0x08` |        1 | `01`                           | Template constant                       |
| `0x10` |        1 | `02`                           | Template constant                       |
| `0x1B` |        2 | `CAPACITY_MB / 16`             | Capacity units, little-endian `u16`     |
| `0x20` |        1 | `01`                           | One protected volume                    |
| `0x28` |        1 | `02`                           | Template constant                       |
| `0x2C` | UTF-16LE | Fixed volume path plus NUL     | Protected volume path                   |

The generated protected volume path is:

```text
\??\Volume{d9b257fc-684e-4dcb-ab79-03cfa2f6b750}
```

It is encoded as UTF-16LE and terminated by a zero `WCHAR`. The remaining bytes in the 4096-byte file stay zero.

## Capacity encoding

The command accepts capacity in MB. The stored unit count is:

```text
units = CAPACITY_MB / 16
```

For example, `4096` MB is stored as `0x0100` at offsets `0x1B` and `0x1C`. The maximum accepted value, `131072` MB (128 GB), is stored as `0x2000`.

The two check bytes use the high and low capacity bytes in the crossed order:

```text
byte[0x00] + byte[0x1C] = 0x24  (8-bit wraparound)
byte[0x03] + byte[0x1B] = 0xF8  (8-bit wraparound)
```

## File handling and driver limits

- A new file is written only after all capacity validation succeeds.
- An existing file is rejected unless `--force` is supplied.
- The parent directory is never created automatically.
- The generated file can be copied to the WinPE image as `Windows\fbwf.cfg`.
- The effective limit also depends on the installed `fbwf.sys`. The configuration format can represent up to 128 GB, but the x86 driver may impose a lower runtime limit.

The fixed volume path is inherited from the reference configuration. If the target WinPE image protects another volume, obtain a configuration containing that volume path first, or update the generator before using the file.

## License

MIT
