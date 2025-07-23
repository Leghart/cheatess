# Cheatess Core

![Build](https://img.shields.io/badge/build-passing-brightgreen)
![License](https://img.shields.io/badge/license-MIT-blue)
![Platform](https://img.shields.io/badge/platform-linux-success)
![Stockfish](https://img.shields.io/badge/engine-stockfish-important)

This is the core logic of the Cheatess project. It allows you to run the application directly from the terminal, play as either white or black, and analyze a live chessboard in real time by monitoring your screen and displaying the best moves based on a preconfigured Stockfish engine.

---

## Table of Contents

- [Requirements](#requirements)
- [Usage](#usage)
  - [Test Mode](#test-mode)
  - [Game Mode](#game-mode)
- [Recommendations](#recommendations)
- [Contributing](#contributing)

---

# Requirements

- A downloaded version of Stockfish compatible with your operating system (https://stockfishchess.org/download/).
- Necessary packages for OpenCV support. For example, on Ubuntu:
  ```bash
  sudo apt install libopencv-dev libwayland-dev pkg-config libpipewire-0.3-dev libegl-dev
  ```

# Usage

## Arguments

Cheatess Core allows you to customize various runtime parameters using command-line arguments.  
There are several subparsers that group related options: `stockfish`, `monitor`, `engine`, and `imgproc`.

### 🧠 Stockfish

Configure the behavior of the Stockfish engine:

- `--path`, `-p` — Path to the Stockfish executable  
  _(See the [Requirements](#requirements) section for more info)_

- `--elo`, `-e` — Set the ELO rating for the engine (default: `1700`)

- `--skill`, `-s` — Skill level of Stockfish (range: 0–20, default: `20`)

- `--depth`, `-d` — How deep the engine should analyze each move  
  ⚠️ Higher values increase response time (default: `5`)

- `--hash` — Memory usage for Stockfish hash tables in MB (default: `16`)

---

### 🖥️ Monitor

Configure which monitor to use for board detection:

- `--number`, `-n` — Select monitor by index (default: `0`, primary monitor)

---

### ♟️ Engine

Customize how the engine output is displayed in the terminal:

- `--pretty`, `-p` — Use Unicode chess pieces instead of plain letters (default: `false`)

---

### 🧪 Image Processing (`imgproc`)

Fine-tune the parameters related to board and piece recognition:

- `--margin`, `-m` — Margin (in pixels) around each square when extracting pieces (default: `5`)

- `--piece-threshold`, `-p` — Threshold for template matching when detecting pieces (default: `0.1`)

- `--extract-piece-threshold`, `-e` — Threshold used when extracting a piece image from a binary board (default: `127.0`)

- `--board-threshold`, `-b` — Threshold for converting the grayscale board to binary (used in piece detection)

- `--difference-level`, `-d` — Pixel difference level between two board images to detect a move (default: `500`)

## Test mode

Before starting a real game, you should configure the parameters to match the appearance of your chess website (e.g., chess.com or lichess.org).
To launch the configuration interface, run:

```bash
cheatess-core --mode test <params to configuration>
```

This will start an interactive test mode where you can verify whether everything is detected correctly.
If you encounter any issues, adjust the parameters manually until the setup works reliably.
Failing to do so may cause errors during an actual game.

<div align="center"> <img src="images/test_mode.png"/> </div>

## Game mode

To start the game, simply run:

```bash
cheatess-core
```

The program will begin monitoring your screen for chess activity. When it detects a move, it will display the move in the terminal, like in the examples below:

<p align="center"> 
    <img src="images/pos1.png" width="200"/> 
    <img src="images/pos2.png" width="200"/> 
    <img src="images/pos3.png" width="200"/> 
</p>

# Recommendations

For the best performance, use a board and piece design that is clear and easy to recognize.
We recommend using a layout similar to the one shown below:

<div align="center"> <img src="images/chess_board.png" width="300"/> </div>

If your monitor resolution is 1920x1080, the following presets should work well:

- chesscom: (default values): `-m 5 -b 100 -e 127 -p 0.1 -d 500`

- lichess: `imgproc -m 4 -b 80 -e 130 -p 0.08 -d 300`

You can always fine-tune these parameters based on your specific board appearance and
screen settings.

# Contributing

Contributions, bug reports, and suggestions are welcome!
Feel free to open an issue or submit a pull request.
Before submitting, please ensure your code is clean and documented.
