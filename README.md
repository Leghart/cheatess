# ♟️ Cheatess

Cheatess is an application that analyzes a live chessboard (e.g., from online platforms like [Lichess](https://lichess.org) or [Chess.com](https://www.chess.com)) and displays the best moves suggested by the Stockfish engine.

It works by monitoring your screen in real time, detecting board positions, and evaluating them using a configurable Stockfish backend.

---

## Project Structure

This project is divided into multiple Rust crates. Please refer to the individual `README.md` files inside each subcrate for detailed instructions and usage:

- [`cheatess-core`](./cheatess-core/) – core logic for screen reading and Stockfish integration

---

## 📌 Note

To get started quickly, you can begin with the [`cheatess-core`](./cheatess-core/) crate, which contains the terminal-based interface and configuration options.
