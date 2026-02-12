#!/bin/bash
# This script is a wrapper for running the tauri dev command
# It ensures that the necessary environment variables are set

# Set up environment variables if needed
export RUST_BACKTRACE=1

# Run the tauri dev command
npm run tauri dev
