#!/usr/bin/env bash
set -uo pipefail
trap 'exit 0' PIPE

# Advanced df output fuzzer for testing parsers / colorizers.
#
# Usage:
#   ./df-fuzz                # default: -h like output, 40 rows
#   ./df-fuzz -n 200         # 200 rows
#   ./df-fuzz --plain        # plain 1K-blocks output (like df without -h)
#   ./df-fuzz --seed 1234    # deterministic
#   ./df-fuzz --wsl          # bias towards WSL-ish entries (C:\, /mnt/wsl, /lib/modules/...)
#   ./df-fuzz --no-spaces    # never include mountpoints with spaces
#
# Pipe into rgrc:
#   ./df-fuzz -n 80 | rgrc -c df
#   ./df-fuzz --plain -n 80 | rgrc -c df

rows=40
mode="human"   # human | plain
seed=""
bias_wsl=0
allow_spaces=1
rng_state=0
RAND_VAL=0

usage() {
  cat <<'EOF'
Usage: ./df-fuzz.sh [options]

Generate synthetic df-style output for parser/colorizer testing.

Options:
  -n, --rows N     Number of data rows to emit (default: 40)
  --plain          Emit plain 1K-blocks output (df default style)
  -h, --human      Emit human-readable output (default)
  --seed N         Use deterministic RNG seed
  --wsl            Bias filesystem/mount choices toward WSL-ish entries
  --no-spaces      Never include mountpoints with spaces
  --help, -?       Show this help and exit
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -n|--rows) rows="${2:?}"; shift 2 ;;
    --plain) mode="plain"; shift ;;
    -h|--human) mode="human"; shift ;;
    --seed) seed="${2:?}"; shift 2 ;;
    --wsl) bias_wsl=1; shift ;;
    --no-spaces) allow_spaces=0; shift ;;
    --help|-\?) usage; exit 0 ;;
    *) echo "Unknown arg: $1" >&2; exit 2 ;;
  esac
done

# 31-bit LCG RNG so values can span large ranges and remain deterministic.
rng_init() {
  if [[ -n "$seed" ]]; then
    rng_state=$((seed & 0x7fffffff))
  else
    rng_state=$(( ((RANDOM << 16) ^ RANDOM ^ $$ ^ SECONDS) & 0x7fffffff ))
  fi
}

rand_u31() {
  rng_state=$(( (1103515245 * rng_state + 12345) & 0x7fffffff ))
  RAND_VAL=$rng_state
}

rand_range() {
  local max="$1"
  if (( max <= 0 )); then
    RAND_VAL=0
    return
  fi
  rand_u31
  RAND_VAL=$((RAND_VAL % max))
}

pick() { # pick array elements
  local -n arr=$1
  local outvar="$2"
  rand_range "${#arr[@]}"
  printf -v "$outvar" '%s' "${arr[$RAND_VAL]}"
}

chance() { # chance N (0..100)
  local n="$1"
  rand_range 100
  (( RAND_VAL < n ))
}

# Escape spaces like df does: "\040"
escape_mount() {
  local s="$1"
  local outvar="$2"
  s="${s// /\\040}"
  printf -v "$outvar" '%s' "$s"
}

# ---------- Distributions / pools ----------
dev_fs=(
  /dev/sda1 /dev/sda2 /dev/sdb1 /dev/sdc1 /dev/sdd1
  /dev/nvme0n1p1 /dev/nvme0n1p2 /dev/nvme1n1p1
  /dev/mapper/cryptroot /dev/mapper/vg0-root /dev/mapper/vg0-home
  UUID=deadbeef-1234-5678-90ab-cafebabe0001
  LABEL=ROOT LABEL=DATA
)

pseudo_fs=(tmpfs devtmpfs overlay rootfs proc sysfs cgroup2 none)
net_fs=(//nas/share nfsserver:/export/home ceph-mon:/volumes/vol1)
weird_fs=(
  "very-long-filesystem-name-that-tries-to-break-columns-aaaaaaaaaaaaaaaaaaaaaaaaaaaa"
  "/dev/disk/by-id/nvme-Samsung_SSD_990_PRO_2TB_S7ZXYZXYZXYZ-part2"
)

mounts_common=(
  / /boot /boot/efi /home /var /var/log /var/lib /var/lib/docker
  /srv /data /backup /archive /nix /nix/store /run /run/user/1000 /dev/shm
  /mnt/c /mnt/d /mnt/e /mnt/wsl
  /lib/modules/5.15.167.4-microsoft-standard-WSL2
  /usr/lib/wsl/drivers /usr/lib/wsl/lib
)

mounts_spaces=(
  "/mnt/External Drive"
  "/Volumes/My Data"
  "/mnt/backup (old)"
  "/var/lib/kubelet/pods/aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee/volumes/kubernetes.io~projected/very long name"
)

# ---------- Formatting helpers ----------
# Convert bytes -> human-ish with one decimal sometimes.
humanize_kib() {
  # input in KiB (1K blocks are KiB-ish; df -h uses powers of 1024-ish)
  local kib="$1"
  local outvar="$2"
  # Choose units: K, M, G, T, P
  local units=(K M G T P)
  local i=0
  local value="$kib"

  # scale by 1024
  while (( value >= 1024 && i < ${#units[@]}-1 )); do
    value=$((value / 1024))
    ((i++))
  done

  # add some decimals randomly, but keep sane
  if chance 35 && (( kib >= 1024 )); then
    # compute one decimal using integer math:
    # v = kib / 1024^i; rem = fractional in tenths
    local denom=1
    for ((j=0; j<i; j++)); do denom=$((denom * 1024)); done
    local int=$((kib / denom))
    local rem=$(((kib * 10 / denom) - int * 10))
    printf -v "$outvar" '%s' "${int}.${rem}${units[$i]}"
  else
    # plain integer
    local denom=1
    for ((j=0; j<i; j++)); do denom=$((denom * 1024)); done
    printf -v "$outvar" '%s' "$((kib / denom))${units[$i]}"
  fi
}

# Format plain integers like df (1K-blocks etc)
plain_kib() {
  local kib="$1"
  local outvar="$2"
  printf -v "$outvar" '%s' "$kib"
}

emitf() {
  if ! printf "$@" 2>/dev/null; then
    exit 0
  fi
}

# Generate a "filesystem" string with weighted weirdness.
gen_fs() {
  local outvar="$1"
  if (( bias_wsl )); then
    if chance 20; then printf -v "$outvar" '%s' "C:\\"; return; fi
    if chance 15; then printf -v "$outvar" '%s' "D:\\"; return; fi
    if chance 10; then printf -v "$outvar" '%s' "E:\\"; return; fi
    if chance 20; then printf -v "$outvar" '%s' "none"; return; fi
  fi

  if chance 55; then
    pick dev_fs "$outvar"
  elif chance 15; then
    pick net_fs "$outvar"
  elif chance 20; then
    pick pseudo_fs "$outvar"
  else
    pick weird_fs "$outvar"
  fi
}

gen_mount() {
  local outvar="$1"
  local m
  if (( allow_spaces )) && chance 15; then
    pick mounts_spaces m
  else
    pick mounts_common m
  fi
  escape_mount "$m" "$outvar"
}

# Create consistent size/used/avail based on target use%
# Input: usepct 0..100, size_kib baseline
# Output: size used avail (all KiB)
compute_triplet() {
  local usepct="$1"
  local size="$2"
  local size_var="$3"
  local used_var="$4"
  local avail_var="$5"

  # Ensure size is at least 1MiB
  if (( size < 1024 )); then size=1024; fi

  # Used = size * pct / 100
  local used=$(( size * usepct / 100 ))
  local avail=$(( size - used ))

  # Add some "df realism": reserved blocks / rounding can cause off-by-one.
  if chance 20; then
    # clamp tweak within [-2..+2]% of size but keep non-negative
    rand_range 5
    local tweak=$((RAND_VAL - 2))  # -2..+2
    local delta=$(( size * tweak / 100 ))
    if (( used + delta >= 0 && used + delta <= size )); then
      used=$((used + delta))
      avail=$((size - used))
    fi
  fi

  printf -v "$size_var" '%s' "$size"
  printf -v "$used_var" '%s' "$used"
  printf -v "$avail_var" '%s' "$avail"
}

# Generate use% with boundary emphasis.
gen_usepct() {
  local outvar="$1"
  local boundaries=(0 1 7 10 49 52 69 75 79 80 81 85 89 90 91 92 94 95 96 97 99 100)
  if chance 55; then
    rand_range "${#boundaries[@]}"
    printf -v "$outvar" '%s' "${boundaries[$RAND_VAL]}"
  else
    rand_range 101
    printf -v "$outvar" '%s' "$RAND_VAL"
  fi
}

# Choose a size distribution that includes tiny, medium, huge.
gen_size_kib() {
  local outvar="$1"
  # Return KiB
  if chance 10; then
    # tiny
    rand_range 900
    printf -v "$outvar" '%s' $((RAND_VAL + 100))          # 100..999 KiB
  elif chance 35; then
    # medium (MiB..GiB)
    rand_range 900000
    printf -v "$outvar" '%s' $((RAND_VAL + 100000))    # ~100MB..~1GB (in KiB)
  elif chance 45; then
    # large (GiB..TiB)
    rand_range 900000000
    printf -v "$outvar" '%s' $((RAND_VAL + 100000000)) # ~100GB..~1TB (in KiB)
  else
    # huge (TiB..few TiB)
    rand_range 3000000000
    printf -v "$outvar" '%s' $((RAND_VAL + 1000000000)) # ~1TB..~4TB (in KiB)
  fi
}

# Decide if line should be tmpfs-like (to test de-emphasis)
force_tmpfs_line() {
  chance 10
}

# ---------- Print header ----------
rng_init

if [[ "$mode" == "plain" ]]; then
  # mimic df default headers
  emitf "%-15s %12s %12s %12s %4s %s\n" "Filesystem" "1K-blocks" "Used" "Available" "Use%" "Mounted on"
else
  emitf "%-15s %6s %6s %6s %4s %s\n" "Filesystem" "Size" "Used" "Avail" "Use%" "Mounted on"
fi

# ---------- Generate rows ----------
for _ in $(seq 1 "$rows"); do
  gen_usepct usepct

  gen_fs fs
  gen_mount mount

  # force some tmpfs rows (and keep them plausible)
  if force_tmpfs_line; then
    fs="tmpfs"
    # Typical tmpfs sizes
    rand_range 8000000
    base_kib=$((RAND_VAL + 200000)) # ~200MB..~8GB in KiB
  else
    gen_size_kib base_kib
  fi

  compute_triplet "$usepct" "$base_kib" size_kib used_kib avail_kib

  if [[ "$mode" == "plain" ]]; then
    plain_kib "$size_kib" size_out
    plain_kib "$used_kib" used_out
    plain_kib "$avail_kib" avail_out
    emitf "%-15s %12s %12s %12s %3s%% %s\n" \
      "$fs" "$size_out" "$used_out" "$avail_out" "$usepct" "$mount"
  else
    humanize_kib "$size_kib" size_out
    humanize_kib "$used_kib" used_out
    humanize_kib "$avail_kib" avail_out
    emitf "%-15s %6s %6s %6s %3s%% %s\n" \
      "$fs" "$size_out" "$used_out" "$avail_out" "$usepct" "$mount"
  fi
done
