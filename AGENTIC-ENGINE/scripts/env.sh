# Mazingira ya MTECH OS / MBILINYI AKILI. Tumia: `source scripts/env.sh`
#
# Hutunza Rust, target, na modeli NDANI ya /workspace (safu inayodumu kwenye
# container hii). /opt na $HOME ni za muda na hupotea container ikirestart.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.." && pwd)"

export CARGO_HOME="${ROOT}/.cargo"
export RUSTUP_HOME="${ROOT}/.rustup"
# Cargo target huenda kwenye diski kuu (/), ambayo ina nafasi zaidi kuliko
# /workspace. Weka AKILI_TARGET_DIR kubadilisha.
export CARGO_TARGET_DIR="${AKILI_TARGET_DIR:-/tmp/akili-target}"
export PATH="${CARGO_HOME}/bin:${HOME}/.local/bin:${PATH}"

# Modeli: 3B kwa majaribio/uthibitisho, 7B kwa uzalishaji.
export AKILI_MODEL="${AKILI_MODEL:-${ROOT}/models/Qwen2.5-VL-3B-Instruct}"
export AKILI_MODEL_PROD="${ROOT}/models/2NYI"
export AKILID_BIN="${CARGO_TARGET_DIR}/debug/akilid"

export RUSTFLAGS="${RUSTFLAGS:--C target-cpu=native}"
