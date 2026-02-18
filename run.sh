#!/bin/bash

# Run script for Kokoro OpenAI Server
# Usage: ./run.sh [args]

set -e

# Default values
export HOST=${HOST:-0.0.0.0}
export PORT=${PORT:-8000}

# Run the server
echo "Starting Kokoro OpenAI Server..."
echo "Host: $HOST:$PORT"
echo ""

exec cargo run --release -- "$@"
