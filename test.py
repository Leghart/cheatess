import time

import cv2
import numpy as np
import pyautogui


def rescale_pieces():
    img = cv2.imread("core/boards/test.png")
    params = img.shape

    print(params)

    x = min((params[0], params[1]))
    side = x // 8
    print(side)

    target_img = cv2.imread("core/chesscom/wn.png", cv2.IMREAD_UNCHANGED)
    new_img = cv2.resize(target_img, (side, side), interpolation=cv2.INTER_AREA)
    cv2.imwrite("resized_knight.png", new_img)


def run():
    st = time.time()
    # Load the chess board and chess piece images
    img_board = cv2.imread("test.png")
    # img_piece = cv2.imread("core/chesscom/bn.png", cv2.IMREAD_UNCHANGED)
    # img_piece = cv2.imread("core/resized/N.png", cv2.IMREAD_UNCHANGED)
    img_piece = cv2.imread("resized_knight.png", cv2.IMREAD_UNCHANGED)

    mask = img_piece[:, :, 3]  # use the inverted transparency channel for mask

    # Convert both images to grayscale
    img_board_gray = cv2.cvtColor(img_board, cv2.COLOR_BGR2GRAY)
    img_piece_gray = cv2.cvtColor(img_piece, cv2.COLOR_BGR2GRAY)
    h, w = img_piece_gray.shape

    # Apply morphological operations to extract the chess piece from the board
    # kernel = np.ones((5, 5), np.uint8)
    # img_piece_mask = cv2.erode(img_piece_gray, kernel, iterations=1)
    # img_piece_mask = cv2.dilate(img_piece_mask, kernel, iterations=1)

    result = cv2.matchTemplate(
        img_board_gray, img_piece_gray, cv2.TM_SQDIFF_NORMED, mask=mask
    )
    min_val, max_val, min_loc, max_loc = cv2.minMaxLoc(result)
    # print(min_val, max_val, min_loc, max_loc)
    # print(f"{min_val} < ")
    while min_val < 0.15:
        # print("found")
        # Draw a rectangle around the matching location
        top_left = min_loc
        bottom_right = (
            top_left[0] + img_piece.shape[1],
            top_left[1] + img_piece.shape[0],
        )
        cv2.rectangle(img_board, top_left, bottom_right, (0, 0, 255), 2)

        # overwrite the portion of the result that has the match:
        h1 = top_left[1] - h // 2
        h1 = np.clip(h1, 0, result.shape[0])

        h2 = top_left[1] + h // 2 + 1
        h2 = np.clip(h2, 0, result.shape[0])

        w1 = top_left[0] - w // 2
        w1 = np.clip(w1, 0, result.shape[1])

        w2 = top_left[0] + w // 2 + 1
        w2 = np.clip(w2, 0, result.shape[1])
        result[h1:h2, w1:w2] = (
            1  # poison the result in the vicinity of this match so it isn't found again
        )

        # look for next match
        min_val, max_val, min_loc, max_loc = cv2.minMaxLoc(result)

    print("TIME", time.time() - st)
    cv2.imwrite("output.png", img_board)

    # Show the result
    # cv2.imshow("Result", img_board)
    # cv2.waitKey(0)
    # cv2.destroyAllWindows()


# # 440.0 219.0 758.0 759.0
# import pyautogui

# img = pyautogui.screenshot(region=(440, 219, 758, 759))
# img.save("test.png")

# rescale_pieces()
while True:
    time.sleep(0.1)
    img = pyautogui.screenshot(region=(440, 219, 758, 759))
    img.save("test.png")
    run()
