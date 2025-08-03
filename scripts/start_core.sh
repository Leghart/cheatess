#!/bin/bash

xhost +local:root 1>/dev/null 2>/dev/null


if [[ "$1" == "-h" || "$1" == "--help" ]]; then
    docker run -it cheatess-core --help
    exit 0
fi

if [[ "$1" == "-m" || "$1" == "--mode" ]]; then
    mode="$2"
    if [[ "$mode" != "test" && "$mode" != "game" ]]; then
        echo "Incorrect mode: $mode. Allowed: test, game"
        exit 1
    fi
    shift 2
else
    echo -e "Usage: $0 -m <test|game> [optional arguments] \n\t or $0 -h for help"
    exit 1
fi

docker run -it \
    -e DISPLAY=$DISPLAY \
    -v /tmp/.X11-unix:/tmp/.X11-unix \
    cheatess-core \
    --mode "$mode" \
    "stockfish -p /usr/games/stockfish" "$@"
