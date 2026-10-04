# Kali-Live Build-Script

_Kali Linux live ISO builder, via `live-build`._

These are the same [build-scripts](https://gitlab.com/kalilinux/build-scripts) that the [Kali team](https://www.kali.org/) uses to generate the official Kali Linux base images, found on [kali.org/get-kali/](https://www.kali.org/get-kali/).

These images can be used to live boot into Kali, from a CD/DVD/Blu-ray/USB/sdCard, as well as offering a basic installation. For more customization during setup, see [kali-installer](https://gitlab.com/kalilinux/build-scripts/kali-installer):
- [kali-installer](https://gitlab.com/kalilinux/build-scripts/kali-installer) uses [Simple-CDD](https://wiki.debian.org/Simple-CDD) _(which is a wrapper for [debian-cd](https://wiki.debian.org/debian-cd))_
- [kali-live](https://gitlab.com/kalilinux/build-scripts/kali-live) uses [live-build](https://live-team.pages.debian.net/live-manual/html/live-manual/index.en.html)

For more information, please see: [kali.org/docs/development/live-build-a-custom-kali-iso/](https://www.kali.org/docs/development/live-build-a-custom-kali-iso/), [kali.org/docs/development/generate-updated-kali-iso/](https://www.kali.org/docs/development/generate-updated-kali-iso/) as well as [kali.org/docs/development/dojo-mastering-live-build/](https://www.kali.org/docs/development/dojo-mastering-live-build/).

_Build [your Kali](https://www.kali.org/docs/introduction/kali-linux-image-overview/), today!_

## Prerequisites

_We recommend building on a Linux-based host, which matches the desired architecture._

There are various ways to get ready to use kali-live build-script. You can either:

- `build.sh` - [Build straight from your machine](#build-from-the-host-kali)
- `build-in-container.sh` - [Build from within a container](#build-from-within-a-container) _(such as Docker or Podman)_ <!-- Should be able to use other alts, but they are untested -->

For `build.sh` to work, it **will require super user access** (e.g. sudo or a rootful container). However, KVM isn't required.

- - -

First install git, and make sure that the repository is cloned locally:

```console
$ sudo apt-get install --no-install-recommends \
    git ca-certificates
$ git clone https://gitlab.com/kalilinux/build-scripts/kali-live.git
$ cd ./kali-live/
```

### Build From The Host (Kali)

To build directly on your host, with `build.sh`, install the build dependencies:

```console
$ sudo apt-get install --no-install-recommends \
    live-build \
    apt-utils \
    qemu-user-binfmt
```
<!-- This should match what is in: [Dockerfile](./Dockerfile) & [kali.org/docs/development/live-build-a-custom-kali-iso/](https://www.kali.org/docs/development/live-build-a-custom-kali-iso/)
     ...with the exception of `qemu-user-binfmt` as it is a HOST requirement, not a container one.
     Other build-scripts install qemu-user-binfmt in their ./Dockerfiles, because their ./build.sh copies the interpreter into the rootfs it ships: find_qemu_static(): $ cp -v ${_qemu} ${SCRATCHPAD_DIR}
-->

_NOTE: Depending on the age of the OS version, you may need to use `qemu-user-static` (legacy) rather than `qemu-user-binfmt` (modern/current/up-to-date)._

_NOTE: `qemu-user-static`/`qemu-user-binfmt` is only required if you are building cross-architecture (e.g. amd64 host, building an arm64 image)._
<!-- > chroot: failed to run command '/bin/true': Exec format error   (debootstrap.log) -->

- - -

Now you can use `./build.sh`, which will build a Kali live ISO straight on your machine.

### Build From The Host (Debian-based, non-Kali)

If you are NOT using Kali as the base OS, you will need to install Kali's `kali-archive-keyring` as well as `live-build` itself, in order to fetch Kali's packages.
We can install the dependencies of `live-build` first, before pulling down Kali's (forked) packages.

```console
$ sudo apt-get install --no-install-recommends \
    debootstrap cpio apt-utils
```
<!-- Alt: $ sudo apt-get satisfy --no-install-recommends live-build -->

- - -

Now for external packages:

```console
$ sudo apt-get install --no-install-recommends \
    wget ca-certificates
$ wget https://http.kali.org/pool/main/k/kali-archive-keyring/kali-archive-keyring_20YY.X_all.deb
$ sudo apt-get install ./kali-archive-keyring_*_all.deb
$
$ wget https://http.kali.org/kali/pool/main/l/live-build/live-build_xxx%2Bkalix_all.deb
$ sudo apt-get install ./live-build_*_all.deb
```
<!-- Alt: $ dpkg -i ./kali-archive-keyring_20YY.X_all.deb
          $ dpkg -i ./live-build_xxx%2Bkalix_all.deb -->

_NOTE: Replace `20YY.X` with the values shown [here, in `kali-archive-keyring`](https://http.kali.org/pool/main/k/kali-archive-keyring/)._

_NOTE: Replace `live-build_xxx%2Bkalix_all.deb` with the values shown [here, in Kali's fork of live-build](https://http.kali.org/kali/pool/main/l/live-build/)._

- - -

Now you can follow [Build From The Host (Kali)](#build-from-the-host-kali) to install build dependencies.

### Build From Within A Container

_We will skip over setting up any container software._

If you prefer to build from within a container, you will need to install and configure either `docker` or `podman` on your machine.

- `docker` requires the user to be added to the Docker group, or using the root account (e.g. `$ sudo ./build-in-container.sh`).
- `podman` has been tested with rootful (e.g. `$ sudo ./build-in-container.sh`). If run rootless, `build-in-container.sh` will attempt to elevate itself via `sudo`, see [Known Limitations](#known-limitations). <!-- lb mounts a fresh sysfs (${PWD}/chroot/sys), which clashes with --network host -->

- - -

`build-in-container.sh` is a wrapper on top of `build.sh`. It detects which OCI-compliant container engine to use, takes care of creating the [container image](./Dockerfile) if missing, and then it starts the container to perform the build from within.

You have three ways to provide the container image:

```console
$ # Option #1 - Automated build (recommended)
$ ./build-in-container.sh
$ ./build-in-container.sh --force   # If you need to rebuild the image from scratch
$
$
$
$ # Option #2 - Manual build
$ docker build -t kali-build/kali-live .
$ # ...OR...
$ podman build -t kali-build/kali-live .
$
$
$
$ # Option #3 - Use the pre-generated
$ docker pull registry.gitlab.com/kalilinux/build-scripts/kali-live:latest
$ docker tag registry.gitlab.com/kalilinux/build-scripts/kali-live:latest kali-build/kali-live
$ # ...OR...
$ podman pull registry.gitlab.com/kalilinux/build-scripts/kali-live:latest
$ podman tag registry.gitlab.com/kalilinux/build-scripts/kali-live:latest kali-build/kali-live
$
$ # ...then point the build at it:
$ ./build-in-container.sh                                                                      # Locally built (automated or manual)
$ IMAGE=registry.gitlab.com/kalilinux/build-scripts/kali-live:latest ./build-in-container.sh   # Pre-generated
```

- - -

If you have both `docker` and `podman` installed, `build-in-container.sh` will default to using `podman`. To change this, prefix `CONTAINER=docker` before `./build-in-container.sh`:

```console
$ CONTAINER=docker ./build-in-container.sh [...]
```

- - -

Now you can use `./build-in-container.sh` (rather than `./build.sh`), to build an image.

## Help

```console
$ ./build.sh --help
Usage: build.sh [OPTIONS]

Build a Kali Linux Live image.

Build options:
  -a, --arch ARCH              Build an image for this architecture (default: amd64)
                               Supported: amd64 arm64 armhf
  -b, --branch BRANCH          Kali branch used to build the image (default: kali-rolling)
                               Supported: kali-rolling kali-dev kali-last-snapshot
  -k, --keep                   Keep intermediary build artifacts
  -m, --mirror URL             Mirror used to build the image (default: http://http.kali.org/kali)
  -o, --output DIR             Output directory (default: /home/kali/kali-live/output)
  -v, --variant VARIANT        Variant of image to build (default: default)
                               Supported: default e17 everything gnome i3 kde large light lxde mate minimal xfce xfce-everything xfce-large xfce-light
  -x, --version VERSION        What to name the image release as (default: rolling)
  -h, --help                   Show this help and exit

Live options:
  -H, --hostname HOSTNAME      Set system host name (default: kali)
  -P, --packages PKGS          Install extra packages (comma/space separated list)

Apt caching proxy:
  Auto-detected: localhost:3142 (apt-cacher-ng), localhost:8000 (squid-deb-proxy).
  If detected and http_proxy is not set, http_proxy is exported automatically.

Supported environment variables:
  http_proxy  HTTP proxy URL, see README.md for details
  DEBUG       Print extra debug output
  Any --flag value can be pre-set via its env var (e.g. ARCH, BUILD_MIRROR, OUT_DIR)

Examples:
  build.sh
  build.sh --branch kali-rolling
  build.sh -b kali-dev -v gnome -a arm64
```

_NOTE: Three flags do not share their variable's name: `--mirror` is `BUILD_MIRROR`, `--hostname` is `KALI_HOSTNAME` (bash already sets `HOSTNAME`) and `--output` is `OUT_DIR`. Every other flag is its own name, upper-cased (e.g. `--branch` is `BRANCH`)._

<!-- _NOTE: Setting a "--flag" value as an environment variable only really works on the host (e.g. `./build.sh`). `build-in-container.sh` passes `OUT_DIR` through, plus its own `HOST_UID`/`HOST_GID`; every other variable you set is dropped._ -->

For more information and/or examples, please see:

- [kali.org/docs/development/live-build-a-custom-kali-iso/](https://www.kali.org/docs/development/live-build-a-custom-kali-iso/)
- [kali.org/docs/development/generate-updated-kali-iso/](https://www.kali.org/docs/development/generate-updated-kali-iso/)
- [kali.org/docs/development/dojo-mastering-live-build/](https://www.kali.org/docs/development/dojo-mastering-live-build/)
- [live-build-config-examples](https://gitlab.com/kalilinux/recipes/live-build-config-examples)
- [kali-preseed-examples](https://gitlab.com/kalilinux/recipes/kali-preseed-examples)
- [kali-installer](https://gitlab.com/kalilinux/build-scripts/kali-installer)

## Build + Custom Values

Use either `build.sh` or `build-in-container.sh`, at your preference.
From this point we will use `build.sh` for brevity.

The default options will build a [Kali rolling](https://www.kali.org/docs/general-use/kali-branches/) image, using [Xfce variant](https://www.kali.org/docs/general-use/switching-desktop-environments/), for AMD64 architecture:

```console
$ sudo ./build.sh
┏━━(Kali Live image)
┃  * Building a Kali Linux Live image for amd64 architecture
┃ # System:
┃  * Build environment is host (bare metal)
┃  * Host architecture is amd64
┃ # Build:
┃  * Branch              : kali-rolling
┃  * Build mirror        : http://http.kali.org/kali/
┃  * Keep temporary files: false
┃  * Output              : /home/kali/kali-live/output/kali-linux-rolling-default-amd64.iso
┃  * Variant             : default
┃  * Version             : rolling
┃ # Live:
┃  * Additional packages :
┃  * Hostname            : kali
┃ # Proxy configuration:
┃  * No apt caching proxy detected
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[...]
```

- - -

Now, we are going to build it from the [last stable release (last snapshot)](https://www.kali.org/docs/general-use/kali-branches/) of Kali (`-b`/`--branch`), using GNOME as the desktop environment (`-v`/`--variant`), and with a custom hostname (`-H`/`--hostname`) and extra packages installed.
We can install additional packages with the `-P`/`--packages` option, on top of whatever the variant normally pulls in: either use it several times (e.g. `-P pkg1 -P pkg2 ...`), or give a comma/space separated value (e.g. `-P "pkg1,pkg2, pkg3 pkg4"`), or a mix of both:

```console
$ sudo ./build.sh -b kali-last-snapshot -v gnome -H kali-gnome -P metasploit-framework,nmap
```

- - -

If we want a very lightweight, headless image with no desktop environment and no default toolset (`-v`/`--variant`), naming this release ourselves (`-x`/`--version`), keep the intermediary build artifacts around afterwards (`-k`/`--keep`), pull from a specific [Kali mirror](https://www.kali.org/docs/community/kali-linux-mirrors/) using `-m`/`--mirror`, as well as more verbose output (`DEBUG=1`):

```console
$ sudo DEBUG=1 ./build.sh --variant minimal --version custom --keep --mirror http://kali.download/kali
```

## Caching Proxy Configuration

When building OS images, it is useful to have a caching mechanism in place, to avoid downloading all the packages from the Internet, again and again.
To this effect, the build script attempts to detect known caching proxies that would be running on the local host. It first refers to the local APT configuration `Acquire::http::Proxy`. If not set, it then tries to detect `apt-cacher-ng` and `squid-deb-proxy` by checking if a service is listening on their default port. This mechanism doesn't work for `approx` (a well-known APT caching proxy), as it's auto-started on demand.

_NOTE: This detection only runs when using the default mirror. If you pass your own `-m/--mirror` value, detection is skipped._

To override this detection, you can export the environment variable `http_proxy` yourself.
For example, if you want to use a proxy that is running on your machine on the port `9876`, use: `export http_proxy=http://127.0.0.1:9876`.
If you want to make sure that no proxy is used, use: `$ http_proxy= ./build.sh`.

Alternatively, you can set up a [local Kali mirror](https://www.kali.org/docs/community/setting-up-a-kali-linux-mirror/).

## Deploy Live Image

The resulting `.iso` is a hybrid image <!-- except armhf, thats img -->: it can be written to a USB/sdCard drive, or burned to a CD/DVD/Blu-ray, to boot Kali Linux live. From within the live session, it's also possible to install Kali to disk. For more customization during that install, see [kali-installer](https://gitlab.com/kalilinux/build-scripts/kali-installer).

### Flash to a USB drive (dd)

```console
$ lsblk
$ sudo dd if=output/kali-linux-rolling-default-amd64.iso of=/dev/<usb-drive> bs=4M status=progress oflag=sync
```

_NOTE: Make sure to use your actual USB device, such as `sdb` instead of `<usb-drive>`._

**CAUTION: This will format the USB drive and erase all its contents!**

#### From Windows (balenaEtcher / Rufus)

You can use [balenaEtcher](https://etcher.balena.io/) or [Rufus](https://rufus.ie/) to flash the image onto the USB drive. Start the tool, select the image file, select the target (the USB drive), then flash.

### Running in QEMU

Being a hybrid ISO, it can also be run directly with QEMU:

```console
$ sudo apt-get install --no-install-recommends \
    qemu-system-x86
$ qemu-system-x86_64 \
    -cdrom output/kali-linux-rolling-default-amd64.iso \
    -enable-kvm \
    -cpu host \
    -m 4096 \
    -smp cores=4
```

## Structure

- `./build.sh` - the main entrypoint _(calls `live-build`'s `lb config`/`lb build`)_
- `./build-in-container.sh` - wraps `./build.sh` inside the build-helper container environment _(using [`./Dockerfile`](./Dockerfile))_
- `./Dockerfile` - the build-helper container _(`live-build`/`apt-utils`)_
- `./auto/` - `live-build`'s own `config`/`clean` hook scripts
- `./kali-config/` - the `live-build` config tree: `common/` (shared) plus one `variant-*/` directory per desktop/edition

## CI Overview

The [`./.gitlab-ci.yml`](./.gitlab-ci.yml) pipeline:

- **`lint`** - `shellcheck`/`yamllint`/`hadolint` against the scripts and Dockerfile
- **`build_push_container`** - builds the build-helper container ([`./Dockerfile`](./Dockerfile)) and [publishes it](https://gitlab.com/kalilinux/build-scripts/kali-live/container_registry)

## Known Limitations

There are a few known limitations of using this build-script:

- [Docker rootless](https://docs.docker.com/engine/security/rootless/) is untested
- `podman` rootless requires `sudo`, see [Build From Within A Container](#build-from-within-a-container)

_If you find something, [let us know](https://gitlab.com/kalilinux/build-scripts/kali-live/-/work_items)!_
