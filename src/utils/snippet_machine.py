class SnippetMachine:
    def __init__(self) -> None:
        self.start_x = None
        self.start_y = None
        self.current_x = None
        self.current_y = None

    def get_frame(self) -> tuple[int]:
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

    def is_frame_set(self) -> bool:
        return all([self.start_x, self.start_y, self.current_x, self.current_y])
