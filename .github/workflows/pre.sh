#!/bin/sh
echo "Running pre-build script"
export PKG_CONFIG_SYSROOT_DIR="/"
apk add --no-cache gtk4.0 glib