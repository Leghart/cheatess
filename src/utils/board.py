import math
from functools import reduce


class InvalidMove(Exception):
    ...


IDX_TO_SIGN = {0: "a", 1: "b", 2: "c", 3: "d", 4: "e", 5: "f", 6: "g", 7: "h"}
BLACK_IDX_TO_SIGN = {7: "a", 6: "b", 5: "c", 4: "d", 3: "e", 2: "f", 1: "g", 0: "h"}

_TMove = tuple[str, int]


def fen_to_list(fen: str) -> list[str]:
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


def get_position_from_idx(idx: int, white_on_move: bool) -> _TMove:
    if white_on_move:
        return (IDX_TO_SIGN[idx % 8], -math.ceil((idx + 1) / 8) + 9)

    return (BLACK_IDX_TO_SIGN[idx % 8], math.floor(idx / 8) + 1)


def fit_board_to_move(
    move1: _TMove, move2: _TMove, board1: list[str], board2: list[str], white_on_move: bool
) -> tuple[_TMove, _TMove]:
    """Return move_from, move_to"""
    move1_piece, move1_idx = move1[0], move1[1]
    move2_idx = move2[1]

    if board1[move1_idx] == move1_piece and board2[move2_idx] == move1_piece and board2[move1_idx] == "":
        move_from = get_position_from_idx(move1_idx, white_on_move)
        move_to = get_position_from_idx(move2_idx, white_on_move)
    else:
        move_from = get_position_from_idx(move2_idx, white_on_move)
        move_to = get_position_from_idx(move1_idx, white_on_move)

    return move_from, move_to


def get_diff_move(board1: list[str], board2: list[str], white_on_move: bool) -> str:
    moved_pieces: list[_TMove] = []
    moved_from_to_save: _TMove = ()
    moved_to_to_save: _TMove = ()

    for idx, (p1, p2) in enumerate(zip(board1, board2), start=0):
        if p1 == p2:
            continue

        moved_pieces.append((p1, idx) if p1 else (p2, idx))

    if len(moved_pieces) <= 1:
        raise InvalidMove

    if len(moved_pieces) == 2:
        moved_from_to_save, moved_to_to_save = fit_board_to_move(
            moved_pieces[0], moved_pieces[1], board1, board2, white_on_move
        )

    if len(moved_pieces) == 3:  # en passant
        white_pawn_occured = reduce(lambda total, sublist: total + (1 if "P" in sublist else 0), moved_pieces, 0)
        black_pawn_occured = reduce(lambda total, sublist: total + (1 if "p" in sublist else 0), moved_pieces, 0)

        if white_pawn_occured < black_pawn_occured:
            pawn_move_idxes = [pos[1] for pos in moved_pieces if pos[0] == "p"]
            moved_from_to_save = get_position_from_idx(pawn_move_idxes[0], white_on_move)
            moved_to_to_save = get_position_from_idx(pawn_move_idxes[1], white_on_move)

        else:
            pawn_move_idxes = sorted([pos[1] for pos in moved_pieces if pos[0] == "P"])[::-1]
            moved_from_to_save = get_position_from_idx(pawn_move_idxes[0], white_on_move)
            moved_to_to_save = get_position_from_idx(pawn_move_idxes[1], white_on_move)

        if not white_on_move:
            moved_from_to_save, moved_to_to_save = moved_to_to_save, moved_from_to_save

    elif len(moved_pieces) == 4:  # castle
        king_move_idxes = [pos[1] for pos in moved_pieces if pos[0] in ("K", "k")]
        if not moved_pieces[0][0] in ("K", "k"):  # long castle
            king_move_idxes = sorted(king_move_idxes)[::-1]

        moved_from_to_save = get_position_from_idx(king_move_idxes[0], white_on_move)
        moved_to_to_save = get_position_from_idx(king_move_idxes[1], white_on_move)

    return f"{moved_from_to_save[0]}{moved_from_to_save[1]}{moved_to_to_save[0]}{moved_to_to_save[1]}"
