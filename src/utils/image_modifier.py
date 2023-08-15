from PIL import Image, ImageDraw


class ImageModifier:
    """Draws circles on the image in places where the piece has been moved and to where it should be."""

    WHITE_BOARD = {"a": 1, "b": 2, "c": 3, "d": 4, "e": 5, "f": 6, "g": 7, "h": 8}
    BLACK_BOARD = {"h": 1, "g": 2, "f": 3, "e": 4, "d": 5, "c": 6, "b": 7, "a": 8}
    SQUARE_SIZE = 50

    def draw(self, from_: str, to_: str, white_on_move: bool) -> Image:
        current_board = Image.open("/home/leghart/projects/cheatess/images/current_board.png")
        draw = ImageDraw.Draw(current_board)
        actual_board = self.WHITE_BOARD if white_on_move else self.BLACK_BOARD

        circ_from_x = int(actual_board[from_[0]])
        circ_from_y = -int(from_[1]) + 9 if white_on_move else int(from_[1])

        circ_to_x = int(actual_board[to_[0]])
        circ_to_y = -int(to_[1]) + 9 if white_on_move else int(to_[1])

        draw.ellipse(
            [
                ((circ_from_x - 1) * self.SQUARE_SIZE, (circ_from_y - 1) * self.SQUARE_SIZE),
                (circ_from_x * self.SQUARE_SIZE, circ_from_y * self.SQUARE_SIZE),
            ],
            width=4,
            outline="red",
        )  # from
        draw.ellipse(
            [
                ((circ_to_x - 1) * self.SQUARE_SIZE, (circ_to_y - 1) * self.SQUARE_SIZE),
                (circ_to_x * self.SQUARE_SIZE, circ_to_y * self.SQUARE_SIZE),
            ],
            width=4,
            outline="red",
        )  # to

        return current_board
