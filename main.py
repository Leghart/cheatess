import glob
import os
import re
import time

import cv2
import numpy as np
import pyautogui

# CHESS_PIECE_DIR = glob.glob("chesscom/*.*")

SHOW_IMAGE = True
EXPORT_IMAGE = True


class Chesscom:
    thresholds = {
        "B": 0.3,  # bishop
        "b": 0.8,  # bishop_black
        "K": 0.2,  # king
        "k": 0.7,  # king_black
        "N": 0.1,  # knight
        "n": 0.5,  # knight_black
        "P": 0.15,  # pawn
        "p": 0.9,  # pawn_black
        "Q": 0.7,  # queen
        "q": 0.7,  # queen_black
        "R": 0.3,  # rook
        "r": 0.3,  # rook_black
    }

    def __str__(self):
        return "chesscom"

    @property
    def region(self):
        return (440, 219, 758, 762)


class Lichess:
    thresholds = {
        "B": 0.2,  # bishop
        "b": 0.3,  # bishop_black
        "K": 0.2,  # king
        "k": 0.2,  # king_black
        "N": 0.1,  # knight
        "n": 0.2,  # knight_black
        "P": 0.15,  # pawn
        "p": 0.3,  # pawn_black
        "Q": 0.2,  # queen
        "q": 0.2,  # queen_black
        "R": 0.2,  # rook
        "r": 0.2,  # rook_black
    }

    def __str__(self):
        return "lichess"

    @property
    def region(self):
        return (568, 218, 720, 720)


def load_pieces(platform: Chesscom | Lichess):
    dir = glob.glob(f"{platform}/*.*")
    chessPieceImages = dict()
    for path in dir:
        baseName = os.path.basename(path)
        fileName = re.search("[\w() -]+?(?=\.)", baseName).group(0)[0]
        pieceImage = cv2.imread(path, cv2.IMREAD_UNCHANGED)
        chessPieceImages[fileName] = (pieceImage, platform.thresholds[fileName])
    return chessPieceImages


def detectPieceOfChess(boardImage: cv2.Mat, chessPieceImages: dict[str, cv2.Mat]):
    start = time.time()

    for piece in chessPieceImages:
        pieceImage = chessPieceImages[piece][0]
        if isinstance(platform, Chesscom) and piece == "p":
            pieceImage = cv2.resize(pieceImage, (43, 43))
        pieceThreshold = chessPieceImages[piece][1]
        pieceName = piece

        boardImageGray = cv2.cvtColor(boardImage, cv2.COLOR_BGR2GRAY)
        pieceImageGray = cv2.cvtColor(pieceImage, cv2.COLOR_BGR2GRAY)

        mask = pieceImage[:, :, 3]
        h, w = pieceImageGray.shape

        result = cv2.matchTemplate(
            boardImageGray, pieceImageGray, cv2.TM_SQDIFF_NORMED, mask=mask
        )
        min_val, max_val, min_loc, max_loc = cv2.minMaxLoc(result)

        while min_val < pieceThreshold:
            top_left = min_loc
            bottom_right = (top_left[0] + w, top_left[1] + h)

            rectangleColor = (0, 250, 50)
            cv2.rectangle(boardImage, top_left, bottom_right, rectangleColor, 2)

            textColor = (255, 0, 0) if pieceName.isupper() else (0, 0, 255)
            textPosition = (top_left[0], top_left[1] + 20)
            cv2.putText(
                boardImage,
                pieceName,
                textPosition,
                cv2.FONT_HERSHEY_SIMPLEX,
                0.7,
                textColor,
                2,
                cv2.LINE_AA,
            )

            h1 = top_left[1] - h // 2
            h1 = np.clip(h1, 0, result.shape[0])

            h2 = top_left[1] + h // 2 + 1
            h2 = np.clip(h2, 0, result.shape[0])

            w1 = top_left[0] - w // 2
            w1 = np.clip(w1, 0, result.shape[1])

            w2 = top_left[0] + w // 2 + 1
            w2 = np.clip(w2, 0, result.shape[1])

            result[h1:h2, w1:w2] = 1

            min_val, max_val, min_loc, max_loc = cv2.minMaxLoc(result)

    print(time.time() - start)
    if SHOW_IMAGE:
        # cv2.imshow("result board", boardImage)
        cv2.imwrite("output.jpg", boardImage)


chesscom = Chesscom()
lichess = Lichess()

platform = chesscom


while True:
    s = time.time()

    img = pyautogui.screenshot(region=platform.region)

    img.save("proxy-board.jpg")

    img = cv2.imread("proxy-board.jpg")
    nimg = cv2.resize(img, (360, 360))
    cv2.imwrite("resized_board.jpg", nimg)

    pieces = load_pieces(platform)

    boardImage = cv2.imread("resized_board.jpg")

    detectPieceOfChess(boardImage, pieces)
    # print(time.time() - s)
    # cv2.waitKey(0)
