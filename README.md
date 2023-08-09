# chessify

# Dataset

Data for training and testig model got from here: https://www.kaggle.com/datasets/koryakinp/chess-positions

If you want to train model to detect FEN's from board images by yourself, you have to download any dataset (it is probably one of the best availables for free). To train model, you can change train-variables like epoches, neural network structure and much more. The only change required is to set a path to directory with train-test dataset (`DATA_PATH`). To start training call method `start_train()` from `src/cnn/train.py`. Your created models are saved in directory `models/custom`.

# Stockfish

## Installation

You have to download stockfish engine from here: https://stockfishchess.org/download/, and type path to place where you store it (Downloaded directory by default) in Chessify settings.

## Usage
