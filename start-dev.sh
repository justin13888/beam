#!/bin/bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

trap 'cleanup' SIGINT SIGTERM
cleanup() {
    echo "Stopping containers..."
    docker stop beam-mysql-dev beam-redis-dev
    docker rm beam-mysql-dev beam-redis-dev
    echo "Containers stopped and removed."
    tmux kill-session -t beamsession
    exit 0
}

cd $SCRIPT_DIR

# Start MySQL in Docker
docker run --replace --name beam-mysql-dev -e MYSQL_ROOT_PASSWORD=beam -e MYSQL_DATABASE=beam -p 3306:3306 -d mysql:8.0-debian

# Start Redis in Docker
docker run --replace --name beam-redis-dev -p 6379:6379 -d redis:7.2-alpine

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
