#!/usr/bin/env bash
#
# $ ./$0
# $ ./$0 [...] --force
# $ CONTAINER=docker ./$0
#

# - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#
## Environment
#

set -euEo pipefail
trap 'cleanup' INT TERM

cd "$( dirname "$( readlink -f "${BASH_SOURCE[0]}" )" )"

# - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#
## Variables
#

CONTAINER="${CONTAINER:-}"
FORCE=0
IMAGE="${IMAGE:-kali-build/kali-live}"
OPTS=()
SUDO=()   # Cannot be rootless

# - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#
## Functions
#

## Output bold only if both stdout/stderr are opened on a terminal
if [ -t 1 ] && [ -t 2 ]; then
  b() { tput bold; echo -n "${*}"; tput sgr0; }
else
  b() { echo -n "${*}"; }
fi

point() { echo " * ${*}"; }
warn()  { echo "WARNING: ${*}" 1>&2; }
fail()  { echo "ERROR: ${*}"   1>&2; exit 1; }

vexec() { b "$ ${*}"; echo; exec "${@}"; }   # Last program in this script should use exec
vrun()  { b "$ ${*}"; echo;      "${@}" <&0 & child_pid=${!}; wait "${child_pid}"; }   # Backgrounded + waited on (rather than a plain foreground call) so cleanup() can kill it if we're interrupted

## Kill vrun()
cleanup() { [ -n "${child_pid:-}" ] && kill -TERM "${child_pid}" 2>/dev/null; return 0; }

build_container() {
  local _cache_args=()
  [ "${FORCE}" -eq 1 ] && _cache_args=(--no-cache)

  if [ "${FORCE}" -eq 1 ] || ! "${SUDO[@]}" "${CONTAINER}" inspect --type image "${IMAGE}" >/dev/null 2>&1; then
    vrun "${SUDO[@]}" "${CONTAINER}" build --network host "${_cache_args[@]}" --tag "${IMAGE}" .
    echo
  fi
}

# - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#
## Parse & validate arguments
#

_passthrough=()
while [ $# -gt 0 ]; do
  case "${1}" in
    --force)  FORCE=1;                shift ;;
    ## Consumed, not forwarded - the --volume below needs OUT_DIR, and it must be absolute
    -o|--output)
              OUT_DIR="$( realpath -m -- "${2:-}" )"; shift 2 ;;
    --output=*)
              OUT_DIR="$( realpath -m -- "${1#*=}" )"; shift ;;
    *)        _passthrough+=("${1}"); shift ;;
  esac
done
set -- "${_passthrough[@]}"
unset _passthrough

if command -v podman >/dev/null 2>&1 && \
  { [ -z "${CONTAINER}" ] || [ "${CONTAINER}" == "podman" ]; }; then
  CONTAINER=podman

  ## We don't want stdout in the journal
  OPTS+=( --log-driver none )

## Rootless podman + --network host + a _fresh_ sysfs mount = kernel-incompatible (EPERM, Operation not permitted) - **rootful container only!**
##   sysfs (/sys) has content that differs per network namespace (netns) - mounting it requires the caller's user namespace to own that netns
##   Rootless podman's user namespace only owns netns it creates itself - however --network host bypasses that, so it doesn't own the host's netns, thus EPERM
##   lb/live-build does a fresh sysfs mount (not a bind-mount) - /usr/lib/live/build/chroot_sysfs: $ mount -t sysfs [...] chroot/sys
##   ...Plus ./build.sh already asks for root use anyway
  [ "$( id -u )" -eq 0 ] || SUDO=(sudo)
elif command -v docker >/dev/null 2>&1 && \
  { [ -z "${CONTAINER}" ] || [ "${CONTAINER}" == "docker" ]; }; then
  CONTAINER=docker
else
  if [ -z "${CONTAINER}" ]; then
    fail "No container engine detected, aborting"
  elif [ "${CONTAINER}" = "podman" ] || [ "${CONTAINER}" = "docker" ]; then
    fail "Requested container engine is not installed: ${CONTAINER}"
  else
    fail "Unknown CONTAINER value: ${CONTAINER}"
  fi
fi

## Permissions & security
OPTS+=(
  ## The list of "--cap-add=[...]" came from `bpftrace` observing cap_capable() during various `./build.sh` runs, all using a --privileged container, with various arguments/options/flags where possible:
  ##   Including kali-linux-default metapackage, custom mirror, custom scratchpad/tmp/output dirs and cross building
  ##   $ sudo bpftrace -e 'kprobe:cap_capable { @[comm, arg2] = count(); }' > /tmp/out.txt
  ## Builds then repeated non-privileged container, starting without any capabilities, adding them in when an issue arises (and documenting why)
  ## The final check was to compare artifacts produced from the "baseline" bare metal build (./build.sh) at the start, with a container (./build-in-container.sh)
  ##   NOTE: but that only lists the ISO - the rootfs is inside live/filesystem.squashfs
  ##   $ bsdtar -tvf output/kali-linux-*.iso
  ## Plus, need to mount both layers and read the capabilities back:
  ##   $ sudo mount -t iso9660 -o ro,loop kali-linux-*.iso /mnt/iso
  ##   $ sudo mount -t squashfs -o ro,loop /mnt/iso/live/filesystem.squashfs /mnt/root
  ##   $ sudo find /mnt/root -printf '%p %M %U:%G %s\n' | sort
  ##   $ sudo getcap -r /mnt/root
  ##   > /mnt/root/usr/bin/fping cap_net_raw=ep
  ##   > /mnt/root/usr/lib/aarch64-linux-gnu/gstreamer1.0/gstreamer-1.0/gst-ptp-helper cap_net_bind_service,cap_net_admin,cap_sys_nice=ep
  ##   > /mnt/root/usr/lib/nmap/nmap cap_net_bind_service,cap_net_admin,cap_net_raw=eip
  ##   $ sudo umount /mnt/root /mnt/iso
  ## Items checked for:
  ##   - Size (compression noise excluded)
  ##   - File count
  ##   - Permissions
  ##   - Owner/group
  ##   - Path
  ##   - Content
  ##   - Capabilities and device nodes
  ## Output that differs between two identical builds is filtered first:
  ##   /etc/ssh/ssh_host_*_key
  ##   /etc/ssl/private/ssl-cert-snakeoil.key
  ##   /var/lib/inetsim/certs/default_key.pem
  ##   /var/lib/texmf/web2c/pdftex/latex.fmt
  ##   /var/lib/texmf/web2c/pdftex/pdflatex.fmt
  ##   /var/lib/texmf/web2c/luahbtex/luahbtex.fmt
  ##   /var/lib/apt/lists/..._Packages
  ##   /var/lib/command-not-found/commands.db
  ##   /var/lib/plocate/plocate.db
  ##   /boot/initrd.img-*
  ##   /usr/share/.../empire/server/data/empire-priv.key
  ##   /usr/share/.../empire/server/data/empire-chain.pem
  --cap-drop=ALL

  --cap-add=CHOWN                       # > E: Tried to extract package, but tar failed. Exit...
                                        # > chown: changing ownership of '[...]/kali-linux-last_snapshot-default-arm64.iso.log': Operation not permitted

  --cap-add=DAC_OVERRIDE                # > cp: cannot create directory 'config': Permission denied
                                        # > tee: '[...]/kali-linux-last_snapshot-default-arm64.iso.log': Permission denied

  --cap-add=FOWNER                      # > cp: preserving permissions for 'cache/bootstrap/run/systemd/netif': Operation not permitted
                                        # > E: An unexpected failure occurred, exiting...

  --cap-add=FSETID                      # Builds, but the artifact loses 21 of its 23 setgid bits
                                        #   /usr/lib/xorg/Xorg.wrap, /usr/local/lib/python2.7

  --cap-add=MKNOD                       # > gpg: Fatal: failed to open '/dev/null': Permission denied
                                        # > old openvas-scanner package postinst maintainer script subprocess failed with exit status 2
                                        # > E: Sub-process /usr/bin/dpkg returned an error code (1)

  --cap-add=SETFCAP                     # > unable to set CAP_SETFCAP effective capability: Operation not permitted
                                        # > Setcap failed on gst-ptp-helper, falling back to setuid
                                        # Builds, but file capabilities are silently dropped, two fall back to setuid:
                                        #   /usr/bin/fping, /usr/lib/*/gstreamer1.0/gstreamer-1.0/gst-ptp-helper
                                        #   /usr/lib/nmap/nmap has no fallback, so it just loses its capability

  --cap-add=SETGID                      # > E: setegid 65534 failed - setegid (1: Operation not permitted)
                                        # > E: Method gave invalid 400 URI Failure message: Failed to setgroups - setgroups (1: Operation not permitted)

  --cap-add=SETUID                      # > E: seteuid 42 failed - seteuid (1: Operation not permitted)
                                        # > E: Method gave invalid 400 URI Failure message: Failed to set new user ids - setresuid (1: Operation not permitted)

  --cap-add=SYS_ADMIN                   # > mount: /build/chroot/dev/pts: permission denied.
                                        # > mount: /build/chroot/proc: permission denied.

  --cap-add=SYS_CHROOT                  # > W: Failure trying to run: chroot "/build/chroot" /bin/true
                                        # > W: See /build/chroot/debootstrap/debootstrap.log for details

  --security-opt apparmor=unconfined    # > mount: /build/chroot/dev/pts: devpts-live already mounted or mount point busy.
                                        # > mount: /build/chroot/proc: proc-live already mounted or mount point busy.
)

## Core values
OPTS+=(
  --rm

  --network host   # Needed to reach detect_apt_caching_proxy()'s proxy, or a LAN --mirror   # podman will not build without it, docker builds unchanged

  --volume "${PWD}:/build"   # Will not build without it: --workdir /build is empty and the entrypoint is not found   # Alt: ${PWD}:/srv or ${PWD}:/recipes   # Alt: --mount type=bind,source=${PWD},destination=/recipes
  --workdir /build

  ## This is for ./build.sh, so it can hand ${OUT_DIR} back to the invoking user (matters more if ran with sudo)
  --env "HOST_UID=${SUDO_UID:-$( id -u )}"
  --env "HOST_GID=${SUDO_GID:-$( id -g )}"
)

## If stdin is an option, use tty
if [ -t 0 ]; then
  OPTS+=(
    --interactive
    --tty
  )
fi

## Output artifacts location
if [ -n "${OUT_DIR:-}" ]; then
  mkdir -pv "${OUT_DIR}"
  OPTS+=(
    --volume "${OUT_DIR}:${OUT_DIR}"
    --env "OUT_DIR=${OUT_DIR}"
  )
fi

if [ -n "${DEBUG:-}" ]; then
  OPTS+=(
    --env "DEBUG=${DEBUG}"
  )
fi

# - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#
## Build
#

build_container

vexec "${SUDO[@]}" "${CONTAINER}" run "${OPTS[@]}" "${IMAGE}" /build/build.sh "${@}"
