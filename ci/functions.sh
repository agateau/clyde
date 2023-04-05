die() {
    echo "Error: $*" >&2
    exit 1
}

# defines ARCH, EXE_DIR, EXE_EXT, and OS_NAME
# optionally defines and exports CARGO_BUILD_TARGET
init_system_vars() {
    ARCH=$(uname -m)

    EXE_EXT=""

    local out
    out=$(uname)

    case "$out" in
    Linux)
        OS_NAME=linux
        export CARGO_BUILD_TARGET="$ARCH-unknown-linux-musl"
        ;;
    Darwin)
        OS_NAME=macos
        ;;
    MINGW*|MSYS*)
        OS_NAME=windows
        EXE_EXT=".exe"
        ;;
    *)
        die "Unknown OS. uname printed '$out'"
        ;;
    esac

    # If $CARGO_BUILD_TARGET is defined it must be included in the exe dir
    EXE_DIR=target/${CARGO_BUILD_TARGET:-}/release
}
