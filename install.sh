#!/bin/sh
set -eu

REPO="samb-t/fora"
BINARY="fora"

#region logging
if [ "${FORA_DEBUG-}" = "true" ] || [ "${FORA_DEBUG-}" = "1" ]; then
  debug() { echo "[debug] $*" >&2; }
else
  debug() { :; }
fi

if [ "${FORA_QUIET-}" = "1" ] || [ "${FORA_QUIET-}" = "true" ]; then
  info() { :; }
else
  info() { echo "fora: $*" >&2; }
fi

error() {
  echo "fora: error: $*" >&2
  exit 1
}
#endregion

#region platform detection
get_os() {
  os="$(uname -s)"
  case "$os" in
    Linux)  echo "linux" ;;
    Darwin) echo "macos" ;;
    MINGW*|MSYS*|CYGWIN*) echo "windows" ;;
    *) error "unsupported OS: $os" ;;
  esac
}

get_arch() {
  arch="$(uname -m)"
  case "$arch" in
    x86_64|amd64) echo "x86_64" ;;
    aarch64|arm64) echo "aarch64" ;;
    *) error "unsupported architecture: $arch" ;;
  esac
}

get_target() {
  os="$1"
  arch="$2"
  case "${os}-${arch}" in
    linux-x86_64)   echo "x86_64-unknown-linux-gnu" ;;
    linux-aarch64)  echo "aarch64-unknown-linux-gnu" ;;
    macos-x86_64)   echo "x86_64-apple-darwin" ;;
    macos-aarch64)  echo "aarch64-apple-darwin" ;;
    windows-x86_64) echo "x86_64-pc-windows-msvc" ;;
    *) error "unsupported platform: ${os}-${arch}" ;;
  esac
}
#endregion

#region download helpers
download() {
  url="$1"
  output="$2"
  if command -v curl >/dev/null 2>&1; then
    debug "curl -fsSL -o $output $url"
    curl -fsSL -o "$output" "$url"
  elif command -v wget >/dev/null 2>&1; then
    debug "wget -qO $output $url"
    wget -qO "$output" "$url"
  else
    error "curl or wget is required but neither is installed"
  fi
}

fetch() {
  url="$1"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO - "$url"
  else
    error "curl or wget is required but neither is installed"
  fi
}
#endregion

install_fora() {
  os="$(get_os)"
  arch="$(get_arch)"
  target="$(get_target "$os" "$arch")"
  install_dir="${FORA_INSTALL_DIR:-$HOME/.local/bin}"

  info "detected platform: ${os}-${arch} (${target})"

  # Determine version
  if [ -n "${FORA_VERSION-}" ]; then
    version="$FORA_VERSION"
    info "installing version: $version"
  else
    info "fetching latest release..."
    version="$(fetch "https://api.github.com/repos/${REPO}/releases/latest" \
      | grep '"tag_name"' \
      | sed 's/.*"tag_name" *: *"\([^"]*\)".*/\1/')"
    if [ -z "$version" ]; then
      error "could not determine latest version. Set FORA_VERSION to install a specific version."
    fi
    info "latest version: $version"
  fi

  # Determine asset name and URL
  if [ "$os" = "windows" ]; then
    asset="fora-${target}.zip"
  else
    asset="fora-${target}.tar.gz"
  fi
  download_url="https://github.com/${REPO}/releases/download/${version}/${asset}"

  # Download
  tmp_dir="$(mktemp -d)"
  trap 'rm -rf "$tmp_dir"' EXIT
  info "downloading ${asset}..."
  download "$download_url" "${tmp_dir}/${asset}"

  # Extract
  cd "$tmp_dir"
  if [ "$os" = "windows" ]; then
    if command -v unzip >/dev/null 2>&1; then
      unzip -q "$asset"
    elif command -v 7z >/dev/null 2>&1; then
      7z x -y "$asset" >/dev/null
    else
      error "unzip or 7z is required to extract the archive"
    fi
    binary_name="fora.exe"
  else
    tar xzf "$asset"
    binary_name="fora"
  fi

  # Install
  mkdir -p "$install_dir"
  mv "$binary_name" "${install_dir}/${binary_name}"
  chmod +x "${install_dir}/${binary_name}"

  info "installed ${BINARY} to ${install_dir}/${binary_name}"

  # Check if install dir is on PATH
  case ":${PATH}:" in
    *":${install_dir}:"*) ;;
    *)
      info ""
      info "WARNING: ${install_dir} is not on your PATH."
      info "Add it by running one of the following:"
      info ""
      case "${SHELL-}" in
        */bash)
          info "  echo 'export PATH=\"${install_dir}:\$PATH\"' >> ~/.bashrc && source ~/.bashrc"
          ;;
        */zsh)
          info "  echo 'export PATH=\"${install_dir}:\$PATH\"' >> ~/.zshrc && source ~/.zshrc"
          ;;
        */fish)
          info "  fish_add_path ${install_dir}"
          ;;
        *)
          info "  export PATH=\"${install_dir}:\$PATH\""
          ;;
      esac
      ;;
  esac
}

install_fora
info ""
info "fora installed successfully! Run 'fora --help' to get started."
