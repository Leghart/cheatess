import math
from dataclasses import dataclass
from typing import Literal, cast


@dataclass
class MovedPiece:
    idx: int
    piece: str


class InvalidMove(Exception):
    ...


_TPiece = Literal["p", "r", "n", "b", "k", "q", "P", "R", "N", "B", "K", "Q"]
_TIdx = Literal[1]
_TIdx.__dict__["__args__"] = list(range(64))
_TCharPosition = Literal["a", "b", "c", "d", "e", "f", "g", "h"]
_TIntPosition = Literal[1, 2, 3, 4, 5, 6, 7, 8]
_TPostMove = tuple[_TCharPosition, _TIntPosition]

IDX_TO_SIGN: dict[int, str] = {0: "a", 1: "b", 2: "c", 3: "d", 4: "e", 5: "f", 6: "g", 7: "h"}
BLACK_IDX_TO_SIGN: dict[int, str] = {
    7: "a",
    6: "b",
    5: "c",
    4: "d",
    3: "e",
    2: "f",
    1: "g",
    0: "h",
}


def fen_to_list(fen: str) -> list[str]:
    """Changes FEN's from string to splitted string as list."""
    assert len(fen.split()) == 1, "FEN has to have only pieces representation!"
    _list = []
    for sign in fen:
        if sign == "/":
            continue

        elif sign.isdigit():
            for _ in range(int(sign)):
                _list.append("")
        else:
            _list.append(sign)

    return _list


def get_position_from_idx(idx: int, white_on_move: bool) -> _TPostMove:
    """Changes field position index to common representation (a8, e4 etc)."""

    if white_on_move:
        char_position = IDX_TO_SIGN[idx % 8]
        int_position = -math.ceil((idx + 1) / 8) + 9

    else:
        char_position = BLACK_IDX_TO_SIGN[idx % 8]
        int_position = math.floor(idx / 8) + 1

    return (cast(_TCharPosition, char_position), cast(_TIntPosition, int_position))


def determine_move(
    moved_from: MovedPiece, moved_to: MovedPiece, board1: list[str], board2: list[str], white_on_move: bool
) -> tuple[_TPostMove, _TPostMove]:
    """Determines a move by comparing the moved pieces and boards before and after the move."""
    if (
        board1[moved_from.idx] == moved_from.piece
        and board2[moved_to.idx] == moved_from.piece
        and board2[moved_from.idx] == ""
    ):
        move_from = get_position_from_idx(moved_from.idx, white_on_move)
        move_to = get_position_from_idx(moved_to.idx, white_on_move)
    else:
        move_from = get_position_from_idx(moved_to.idx, white_on_move)
        move_to = get_position_from_idx(moved_from.idx, white_on_move)

    return move_from, move_to


def get_diff_move(board1: list[str], board2: list[str], white_on_move: bool) -> str:
    moved_pieces: list[MovedPiece] = []

    for idx, (p1, p2) in enumerate(zip(board1, board2), start=0):
        if p1 == p2:
            continue

        moved_pieces.append(MovedPiece(piece=p1, idx=idx) if p1 else MovedPiece(piece=p2, idx=idx))

    if len(moved_pieces) <= 1:
        raise InvalidMove

    if len(moved_pieces) == 2:
        moved_from_to_save, moved_to_to_save = determine_move(
            moved_pieces[0], moved_pieces[1], board1, board2, white_on_move
        )

    if len(moved_pieces) == 3:  # en passant
        white_pawn_occured = 0
        black_pawn_occured = 0

        for piece in moved_pieces:
            if piece.piece == "P":
                white_pawn_occured += 1
            if piece.piece == "p":
                black_pawn_occured += 1

        if white_pawn_occured < black_pawn_occured:
            pawn_move_idxes = [pos.idx for pos in moved_pieces if pos.piece == "p"]
            moved_from_to_save = get_position_from_idx(pawn_move_idxes[0], white_on_move)
            moved_to_to_save = get_position_from_idx(pawn_move_idxes[1], white_on_move)

        else:
            pawn_move_idxes = sorted([pos.idx for pos in moved_pieces if pos.piece == "P"])[::-1]
            moved_from_to_save = get_position_from_idx(pawn_move_idxes[0], white_on_move)
            moved_to_to_save = get_position_from_idx(pawn_move_idxes[1], white_on_move)

        if not white_on_move:
            moved_from_to_save, moved_to_to_save = moved_to_to_save, moved_from_to_save

    elif len(moved_pieces) == 4:  # castle
        king_move_idxes = [pos.idx for pos in moved_pieces if pos.piece in ("K", "k")]
        if not moved_pieces[0].piece in ("K", "k"):  # long castle
            king_move_idxes = sorted(king_move_idxes)[::-1]

        moved_from_to_save = get_position_from_idx(king_move_idxes[0], white_on_move)
        moved_to_to_save = get_position_from_idx(king_move_idxes[1], white_on_move)

    return f"{moved_from_to_save[0]}{moved_from_to_save[1]}{moved_to_to_save[0]}{moved_to_to_save[1]}"
