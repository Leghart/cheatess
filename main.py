import glob
import os
import time

import cv2
import pyautogui as pyautogui

# img = pyautogui.screenshot(region=(440, 219, 758, 759))
# img.save("board3.png")


def match_template_limited(
    gray_chessboard,
    gray_figure,
    threshold=0.8,
    board_width=759,
    board_height=758,
):
    ss = time.time()
    result = cv2.matchTemplate(gray_chessboard, gray_figure, cv2.TM_CCOEFF_NORMED)
    # print("template: ", time.time() - ss)
    # mask = np.ones(result.shape, dtype=np.uint8)
    square_width = board_width // 8
    square_height = board_height // 8

    unique_points = []
    # used_squares = set()
    a = 0
    while True:
        min_val, max_val, min_loc, max_loc = cv2.minMaxLoc(result)

        if max_val < threshold:
            break

        x, y = max_loc
        # col = x // square_width
        # row = y // square_height

        # if (row, col) in used_squares:
        a += 1
        #     result[y : y + square_height, x : x + square_width] = 0
        #     continue

        # used_squares.add((row, col))
        unique_points.append((x, y))

        x_min = max(0, x - square_width // 2)
        y_min = max(0, y - square_height // 2)
        x_max = min(result.shape[1], x + square_width // 2)
        y_max = min(result.shape[0], y + square_height // 2)

        result[y_min:y_max, x_min:x_max] = 0
    # print(a)
    return unique_points


def run(brd, pcs):
    chessboard = cv2.imread(brd, cv2.IMREAD_COLOR)
    figure_paths = glob.glob(f"{pcs}/*.png")
    w, h, _ = chessboard.shape
    threshold = 0.75

    board = [[None for _ in range(8)] for _ in range(8)]
    square_w = w / 8
    square_h = h / 8
    a = 0
    match_times = []
    for fig_path in figure_paths:
        figure = cv2.imread(fig_path, cv2.IMREAD_UNCHANGED)
        figure_name = os.path.basename(fig_path).replace(".png", "")

        gray_figure = cv2.cvtColor(figure, cv2.COLOR_BGR2GRAY)
        gray_chessboard = cv2.cvtColor(chessboard, cv2.COLOR_BGR2GRAY)

        ss = time.time()
        loc = match_template_limited(gray_chessboard, gray_figure, threshold)
        match_times.append(time.time() - ss)
        for pt in loc:
            col = int(pt[0] // square_w)
            row = int(pt[1] // square_h)

            board[row][col] = True

            cv2.rectangle(
                chessboard,
                pt,
                (pt[0] + gray_figure.shape[1], pt[1] + gray_figure.shape[0]),
                (0, 0, 255),
                1,
            )

            # cv2.putText(
            #     chessboard,
            #     figure_name,
            #     (pt[0], pt[1] + 10),
            #     cv2.FONT_HERSHEY_SIMPLEX,
            #     1,
            #     (0, 255, 0),
            #     1,
            #     cv2.LINE_AA,
            # )

        # print(figure_name, a)

    cv2.imwrite("output.png", chessboard)
    # cv2.imshow("result", chessboard_copy)
    # cv2.waitKey(0)
    # print("match times: ", sum(match_times))


while True:
    st = time.time()

    # for _ in range(1):
    img = pyautogui.screenshot(region=(440, 219, 758, 759))
    img.save("board3.png")
    run("board3.png", "npieces")
    # time.sleep(0.2)
    print("time: ", time.time() - st)
