#!/usr/bin/env bash
set -e pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# Print in colors - 1=green, 2=red, other=neutral
# e.g. log_print 0 "All is great"
log_print() {
    if [[ $1 == 1 ]]; then
        echo -e "${GREEN}${2}${NC}"
        elif [[ $1 == 2 ]]; then
        echo -e "${RED}${2}${NC}"
    else
        echo -e "${2}"
    fi
}

# Function used to check if utilities are available
require() {
    if ! hash "$1" &>/dev/null; then
        log_print 2 "'$1' not found in PATH. This is required for this script to work."
        exit 1
    fi
}

require curl
require tar
require uname

log_print 0 "Checking for the latest version of runtime.land..."

get_arch() {
    ARCH=$(uname -m)
    case $ARCH in
        amd64) ARCH="amd64" ;;
        x86_64) ARCH="amd64" ;;
        aarch64) ARCH="aarch64" ;;
        riscv64) ARCH="riscv64" ;;
        arm64) ARCH="aarch64" ;; # This is for the macOS M1 ARM chips
        *)
            log_print 2 "The system architecture (${ARCH}) is not yet supported by this installation script."
            exit 1
        ;;
    esac
    # echo "ARCH = $ARCH"
}

get_os() {
    OS=$(uname | tr '[:upper:]' '[:lower:]')
    case "$OS" in
        darwin) OS='macos' ;;
        linux) OS='linux' ;;
        freebsd) OS='freebsd' ;;
        # mingw*) OS='windows';;
        # msys*) OS='windows';;
        *)
            log_print 2 "The OS (${OS}) is not supported by this installation script."
            exit 1
        ;;
    esac
}

get_latest_release() {
    curl --silent "https://api.github.com/repos/fuxiaohei/runtime.land/releases/latest" | \
    grep tag_name | \
    cut -d '"' -f 4
}

get_arch
get_os
VERSION=$(get_latest_release)
# if VERSION is empty, then exit
if [ -z "${VERSION-}" ]; then
    log_print 2 "Failed to get the latest version of runtime.land"
    exit 1
fi

log_print 1 "Fetching latest version of runtime.land for ${OS}-${ARCH}, version ${VERSION}"


download_release() {
    local tmpdir=$(mktemp -d)
    local filename="land-cli-${VERSION}-${OS}-${ARCH}.tar.gz"
    local download_file="$tmpdir/$filename"
    local archive_url="https://github.com/fuxiaohei/runtime.land/releases/download/${VERSION}/$filename"
    
    log_print 1 "Downloading $archive_url"
    curl --progress-bar --show-error --location --fail "$archive_url" \
    --output "$download_file"
    DOWNLOAD_FILE="$download_file"
}

# download_release
download_release

# extract_release
INSTALL_DIR="$HOME/.runtimeland"
extract_release() {
    log_print 1 "Extracting binary to $INSTALL_DIR"
    mkdir -p "$INSTALL_DIR"
    tar -xzf "$DOWNLOAD_FILE" -C "$INSTALL_DIR"
}
extract_release

# get the shell type
SHELLTYPE="$(basename "/$SHELL")"
detect_profile() {
    
    local DETECTED_PROFILE
    DETECTED_PROFILE=''
    
    if [ "$SHELLTYPE" = "bash" ]; then
        if [ -f "$HOME/.bashrc" ]; then
            DETECTED_PROFILE="$HOME/.bashrc"
            elif [ -f "$HOME/.bash_profile" ]; then
            DETECTED_PROFILE="$HOME/.bash_profile"
        fi
        elif [ "$SHELLTYPE" = "zsh" ]; then
        DETECTED_PROFILE="$HOME/.zshrc"
        elif [ "$SHELLTYPE" = "fish" ]; then
        DETECTED_PROFILE="$HOME/.config/fish/config.fish"
    fi
    
    if [ -z "$DETECTED_PROFILE" ]; then
        if [ -f "$HOME/.profile" ]; then
            DETECTED_PROFILE="$HOME/.profile"
            elif [ -f "$HOME/.bashrc" ]; then
            DETECTED_PROFILE="$HOME/.bashrc"
            elif [ -f "$HOME/.bash_profile" ]; then
            DETECTED_PROFILE="$HOME/.bash_profile"
            elif [ -f "$HOME/.zshrc" ]; then
            DETECTED_PROFILE="$HOME/.zshrc"
            elif [ -f "$HOME/.config/fish/config.fish" ]; then
            DETECTED_PROFILE="$HOME/.config/fish/config.fish"
        fi
    fi
    
    if [ ! -z "$DETECTED_PROFILE" ]; then
        echo "$DETECTED_PROFILE"
    fi
}

USER_PROFILE="${PROFILE:-$(detect_profile)}"

build_profile_content() {
    if [ "$SHELLTYPE" = "fish" ]; then
        # fish uses a little different syntax to modify the PATH
    cat <<END_FISH_SCRIPT

# Runtime.land
set -gx RUNTIMELAND_HOME "$INSTALL_DIR"
string match -r ".runtimeland" "\$PATH" > /dev/null; or set -gx PATH "\$RUNTIMELAND_HOME/bin" \$PATH

END_FISH_SCRIPT
    else
        # bash and zsh
    cat <<END_BASH_SCRIPT

# Runtime.land
export RUNTIMELAND_HOME="$INSTALL_DIR"
export PATH="\$RUNTIMELAND_HOME:\$PATH"

END_BASH_SCRIPT
    fi
}
PROFILE_CONTENT=$(build_profile_content)

update_profile(){
    if [ -z "${USER_PROFILE-}" ] ; then
        echo "1"
        echo "No user profile found."
        echo "Tried \$PROFILE ($PROFILE), ~/.bashrc, ~/.bash_profile, ~/.zshrc, ~/.profile, and ~/.config/fish/config.fish."
        echo ''
        echo "You can either create one of these and try again or add this to the appropriate file:"
        echo "$PROFILE_CONTENT"
        return 1
    else
        if ! command grep -qc 'RUNTIMELAND_HOME' "${USER_PROFILE}"; then
            log_print 1 "Adding runtime.land CLI to profile (${USER_PROFILE})"
            echo "$PROFILE_CONTENT" >> "$USER_PROFILE"
        else
            log_print 1 "Your profile ($USER_PROFILE) already add Runtime.land and has not been changed."
        fi
    fi
}

update_profile

log_print 1 "Finished installation. Open a new terminal to start using Runtime.land land-cli!"
