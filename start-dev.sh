#!/bin/bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

trap 'cleanup' SIGINT SIGTERM
cleanup() {
    echo "Stopping containers..."
    docker stop beam-postgres-dev 
    docker rm beam-postgres-dev
    echo "Containers stopped and removed."
    tmux kill-session -t beamsession
    exit 0
}

cd $SCRIPT_DIR

# Start Postgres in Docker
docker run --replace --name beam-postgres-dev -e POSTGRES_USER=beam -e POSTGRES_PASSWORD=beam -e POSTGRES_DB=beam -p 5432:5432 -d postgres:16

# Start tmux session
tmux new-session -d -s beamsession -n 'beam-web'

# Start beam-web
tmux send-keys -t beamsession 'cd '"$SCRIPT_DIR/beam-web" C-m
tmux send-keys -t beamsession 'pnpm dev' C-m

# Create new tmux tab
tmux new-window -t beamsession:1 -n 'beam-api'

# Start beam-api
tmux send-keys -t beamsession 'cd '"$SCRIPT_DIR/beam-api" C-m
tmux send-keys -t beamsession 'bun dev' C-m

# Attach to tmux session
tmux attach -t beamsession

cleanup
