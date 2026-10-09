# balatro-storage

Find Balatro's real save files and copy them in and out, written in Rust.
Supports macOS Apple Arcade today.

## Overview

Balatro+ on Apple Arcade stores `meta.jkr` and `profile.jkr` as binary blobs
inside a system plist file, rather than as plain files on disk. `save.jkr` and
`settings.jkr` live in the app's sandboxed support directory.

This tool can extract all file types to a directory you specify, and write
them back when you want to restore. The `.jkr` files are treated as opaque bytes,
use [`balatro-jkr`](../balatro-jkr) to decode them and [`balatro-profile`](../balatro-profile)
to read them as typed data.

## Usage

### List what exists

```
cargo run -p balatro-storage -- list
```

```
profile    meta     profile    save     settings
--------------------------------------------------
1          yes      yes        yes      yes
2          yes      yes        no       yes
3          yes      yes        no       yes
```

A `yes` in the `save` column means there is an active run in progress for that profile.

### Export

```
cargo run -p balatro-storage -- export --profile 1 --out ./backup
```

```
Profile 1 exported:
  meta    → ./backup/1-meta.jkr
  profile → ./backup/1-profile.jkr
  save    → ./backup/1-save.jkr
  settings→ ./backup/settings.jkr
```

Use `--profile all` to export all three profiles in one pass.

### Import

```
cargo run -p balatro-storage -- import --profile 1 --from ./backup
```

The game must not be running when importing. If it is, the command exits with an
error rather than writing files that will be overwritten on quit.

### Path overrides

The plist and game support directory are resolved automatically from
`~/Library/Containers/com.playstack.balatroarcade/`. Both can be overridden if
needed (e.g. pointing at a backup copy of the plist):

```
balatro-storage --plist /path/to/custom.plist export --profile 1 --out ./backup
balatro-storage --game-dir /path/to/game list
```

## Install

```
cargo install --path balatro-storage
```

This puts `balatro-storage` on your PATH so you can run it from anywhere.
