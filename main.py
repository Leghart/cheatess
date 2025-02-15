import glob
import os
import time

import cv2
import numpy as np
import pyautogui as pyautogui

st = time.time()

img = pyautogui.screenshot(region=(440, 219, 758, 759))
img.save("board3.png")
# exit(0)


def match_template_limited(
    gray_chessboard,
    gray_figure,
    threshold=0.8,
    board_width=759,
    board_height=758,
):

    result = cv2.matchTemplate(gray_chessboard, gray_figure, cv2.TM_CCOEFF_NORMED)

    mask = np.ones(result.shape, dtype=np.uint8)

    square_width = board_width // 8
    square_height = board_height // 8

    loc = np.where(result >= threshold)

    unique_points = []
    used_squares = set()

    for pt in zip(*loc[::-1]):
        x, y = pt

        col = x // square_width
        row = y // square_height

        if (row, col) in used_squares:
            continue

        used_squares.add((row, col))
        unique_points.append((x, y))

        x_min = max(0, x - square_width // 2)
        y_min = max(0, y - square_height // 2)
        x_max = min(result.shape[1], x + square_width // 2)
        y_max = min(result.shape[0], y + square_height // 2)

        mask[y_min:y_max, x_min:x_max] = 0
        result = result * mask

    return unique_points


def extract_chess_pieces(image_path, output_folder="npieces"):
    os.makedirs(output_folder, exist_ok=True)
    image = cv2.imread(image_path, cv2.IMREAD_UNCHANGED)

    gray = cv2.cvtColor(image, cv2.COLOR_BGR2GRAY)

    # _, thresh = cv2.threshold(gray, 0, 255, cv2.THRESH_BINARY_INV + cv2.THRESH_OTSU)
    _, thresh = cv2.threshold(gray, 40, 255, cv2.THRESH_BINARY)
    # thresh = cv2.Canny(gray, 100, 200)
    cv2.imshow("", thresh)
    cv2.waitKey(0)
    # exit(1)
    # contours, _ = cv2.findContours(thresh, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
    contours, _ = cv2.findContours(thresh, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_TC89_L1)

    for i, contour in enumerate(contours):

        x, y, w, h = cv2.boundingRect(contour)

        figure = image[y : y + h, x : x + w]

        save_path = os.path.join(output_folder, f"figure_{i}.png")
        # cv2.imwrite(save_path, figure)
        # cv2.imshow("", figure)
        # cv2.waitKey(0)


def run(brd, pcs):
    chessboard = cv2.imread(brd, cv2.IMREAD_COLOR)
    figure_paths = glob.glob(f"{pcs}/*.png")
    chessboard_copy = chessboard.copy()
    w, h, _ = chessboard.shape
    threshold = 0.75

    board = [[None for _ in range(8)] for _ in range(8)]
    square_w = w / 8
    square_h = h / 8
    a = 0

    for fig_path in figure_paths:
        figure = cv2.imread(fig_path, cv2.IMREAD_UNCHANGED)
        figure_name = os.path.basename(fig_path).replace(".png", "")

        gray_figure = cv2.cvtColor(figure, cv2.COLOR_BGR2GRAY)
        gray_chessboard = cv2.cvtColor(chessboard, cv2.COLOR_BGR2GRAY)

        result = cv2.matchTemplate(gray_chessboard, gray_figure, cv2.TM_CCOEFF_NORMED)
        loc = np.where(result >= threshold)

        # loc = match_template_limited(gray_chessboard, gray_figure)

        for pt in zip(*loc[::-1]):
            # for pt in loc:
            # if dist(pt, last) < 200:
            # print(f"skip for {figure_name}: {pt}")
            # continue
            col = int(pt[0] // square_w)
            row = int(pt[1] // square_h)
            if board[row][col] is not None:
                a += 1
                continue
            board[row][col] = True
            # break
            cv2.rectangle(
                chessboard_copy,
                pt,
                (pt[0] + gray_figure.shape[1], pt[1] + gray_figure.shape[0]),
                (0, 0, 255),
                1,
            )

            cv2.putText(
                chessboard_copy,
                figure_name,
                (pt[0], pt[1] + 10),
                cv2.FONT_HERSHEY_SIMPLEX,
                1,
                (0, 255, 0),
                1,
                cv2.LINE_AA,
            )
        print(figure_name, a)

    print("time: ", time.time() - st)

    cv2.imshow("result", chessboard_copy)
    cv2.waitKey(0)


# extract_chess_pieces("board1.png")

run("board3.png", "npieces")
