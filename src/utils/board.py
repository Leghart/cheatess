import math
from functools import reduce


class InvalidMove(Exception):
    ...


IDX_TO_SIGN = {0: "a", 1: "b", 2: "c", 3: "d", 4: "e", 5: "f", 6: "g", 7: "h"}


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


def get_position_from_idx(idx: int) -> _TMove:
    return (IDX_TO_SIGN[idx % 8], -math.ceil((idx + 1) / 8) + 9)


def fit_board_to_move(move1: _TMove, move2: _TMove, board1: list[str], board2: list[str]) -> tuple[_TMove, _TMove]:
    """Return move_from, move_to"""
    move1_piece, move1_idx = move1[0], move1[1]
    move2_idx = move2[1]

    if board1[move1_idx] == move1_piece and board2[move2_idx] == move1_piece and board2[move1_idx] == "":
        move_from = get_position_from_idx(move1_idx)
        move_to = get_position_from_idx(move2_idx)
    else:
        move_from = get_position_from_idx(move2_idx)
        move_to = get_position_from_idx(move1_idx)

    return move_from, move_to


def get_diff_move(board1: list[str], board2: list[str]) -> str:
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
        move_from, move_to = fit_board_to_move(moved_pieces[0], moved_pieces[1], board1, board2)
        moved_from_to_save = move_from
        moved_to_to_save = move_to

    if len(moved_pieces) == 3:  # en passant
        white_pawn_occured = reduce(lambda total, sublist: total + (1 if "P" in sublist else 0), moved_pieces, 0)
        black_pawn_occured = reduce(lambda total, sublist: total + (1 if "p" in sublist else 0), moved_pieces, 0)

        if white_pawn_occured < black_pawn_occured:
            pawn_move_idxes = [pos[1] for pos in moved_pieces if pos[0] == "p"]
            moved_from_to_save = get_position_from_idx(pawn_move_idxes[0])
            moved_to_to_save = get_position_from_idx(pawn_move_idxes[1])

        else:
            pawn_move_idxes = sorted([pos[1] for pos in moved_pieces if pos[0] == "P"])[::-1]
            moved_from_to_save = get_position_from_idx(pawn_move_idxes[0])
            moved_to_to_save = get_position_from_idx(pawn_move_idxes[1])

    elif len(moved_pieces) == 4:  # castle
        king_move_idxes = [pos[1] for pos in moved_pieces if pos[0] in ("K", "k")]
        if not moved_pieces[0][0] in ("K", "k"):  # long castle
            king_move_idxes = sorted(king_move_idxes)[::-1]

        moved_from_to_save = get_position_from_idx(king_move_idxes[0])
        moved_to_to_save = get_position_from_idx(king_move_idxes[1])

    return f"{moved_from_to_save[0]}{moved_from_to_save[1]}{moved_to_to_save[0]}{moved_to_to_save[1]}"
