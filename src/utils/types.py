from typing import TypedDict


class TopMoves(TypedDict):
    Move: str
    Centipawn: int | None
    Mate: int | None


class FinalTopMoves(TypedDict):
    move: str
    evaluation: str


class StatisticsDict(TypedDict):
    wdl_stats: list[int]
    top_moves: list[FinalTopMoves]
    best_move: str
