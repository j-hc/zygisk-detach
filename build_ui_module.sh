#!/bin/bash
set -e

echo "[*] Setting up environment for building zygisk-detach module..."

# Check and setup Node.js/NPM (for WebUI)
if ! command -v node &> /dev/null; then
  echo "[-] node/npm not found. Installing via Homebrew..."
  if command -v brew &> /dev/null; then
    brew install node
  else
    echo "ERROR: Homebrew not found. Please install Homebrew first (brew.sh) or manually install Node.js"
    exit 1
  fi
else
  echo "[+] node: $(node -v) found"
fi

# Check and setup Rust/Cargo (for CLI)
if ! command -v cargo &> /dev/null; then
  echo "[-] cargo not found. Installing rustup..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source "$HOME/.cargo/env"
else
  echo "[+] rust: $(cargo --version) found"
fi

# Check Android NDK 29
NDK_VERSION="29.0.14206865"
NDK_PATH="$HOME/Library/Android/sdk/ndk/$NDK_VERSION"
if [ ! -d "$NDK_PATH" ]; then
  echo "[-] NDK 29 not found at $NDK_PATH"
  echo "Please make sure NDK version $NDK_VERSION is downloaded via Android Studio SDK Manager."
  exit 1
else
  echo "[+] NDK 29 found at $NDK_PATH"
  export ANDROID_NDK_HOME="$NDK_PATH"
fi

echo "----------------------------------------"
echo "[*] Building WebUI..."
cd ksu-webui || exit
npm install
npx parcel build src/index.html --dist-dir ../magisk/webroot --public-url ./
cd ..

echo "----------------------------------------"
echo "[*] Dependencies installed and WebUI built inside magisk/webroot!"
echo "If you wish to fully compile the native Zygisk hooks (.so) and CLI rust binaries (.unstripped),"
echo "you will need the project's official make/build files (e.g. Android.mk), which seem to be missing locally."
echo "However, you can now safely copy/replace your updated ksu-webui files into the official zip!"
echo "Done."
