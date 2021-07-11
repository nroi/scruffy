# Scruffy

A pacman cleaning tool without any dependencies to libalpm.

The main purpose of this tool is to be able to clean the pacman cache with systems that don't run ArchLinux
and therefore don't have pacman installed. Existing tools, such as 
[pacman-contrib](https://archlinux.org/packages/community/x86_64/pacman-contrib/),
[pkgcacheclean](https://aur.archlinux.org/packages/pkgcacheclean/) or
[pacleaner](https://aur.archlinux.org/packages/pacleaner/)
require pacman/libalpm and therefore cannot be installed (without much effort) on alternative Linux distros.

In particular, the docker image of [Flexo](https://github.com/nroi/flexo), which is currently based
on Debian, uses this tool.

## Build

Cargo is required to build scruffy:

```
cargo build --release
```

## Usage

The `-h` flag explains all options:

```
./target/release/scruffy -h
```

## Examples

To print all files that are older than the 2 most recent versions in `/var/cache/flexo/pkg`,
use the `--dryrun` (or `-d`) flag in combination with the `--verbose` (or `-v`) flag:

```
./target/release/scruffy -v -d --keep 2 -c /var/cache/flexo/pkg

> community/os/x86_64/0ad-a23.1-9-x86_64.pkg.tar.zst
> community/os/x86_64/abcde-2.9.3-1-any.pkg.tar.xz
> core/os/x86_64/acl-2.2.53-1-x86_64.pkg.tar.xz
> extra/os/x86_64/adobe-source-code-pro-fonts-2.030ro+1.050it-6-any.pkg.tar.zst
> [more files…]
> extra/os/x86_64/zziplib-0.13.70-1-x86_64.pkg.tar.zst
> 
> [dry run] 1852 packages removed (disk space saved: 7.36 GiB)
```


The following will remove all files except for the 3 most recent versions:
```
./target/release/scruffy -d --keep 3 -c /var/cache/flexo/pkg
```
