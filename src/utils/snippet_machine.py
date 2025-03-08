class SnippetMachine:
    """Represents a snippet which user use to designate an area where the board is placed."""

    def __init__(self) -> None:
        self.start_x: int = 0
        self.start_y: int = 0
        self.current_x: int = 0
        self.current_y: int = 0

    def get_frame(self) -> tuple[int, int, int, int]:
        """Convert created snippet frame to unified value."""
        if self.start_x <= self.current_x and self.start_y <= self.current_y:
            return (
                self.start_x,
                self.start_y,
                self.current_x - self.start_x,
                self.current_y - self.start_y,
            )

        elif self.start_x >= self.current_x and self.start_y <= self.current_y:
            return (
                self.current_x,
                self.start_y,
                self.start_x - self.current_x,
                self.current_y - self.start_y,
            )

        elif self.start_x <= self.current_x and self.start_y >= self.current_y:
            return (
                self.start_x,
                self.current_y,
                self.current_x - self.start_x,
                self.start_y - self.current_y,
            )

        elif self.start_x >= self.current_x and self.start_y >= self.current_y:
            return (
                self.current_x,
                self.current_y,
                self.start_x - self.current_x,
                self.start_y - self.current_y,
            )
        else:
            raise KeyError("Unexpected snippet coordinates")

    def is_frame_set(self) -> bool:
        return all([self.start_x, self.start_y, self.current_x, self.current_y])
