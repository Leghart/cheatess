use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use subprocess::{Popen, PopenConfig, Redirection};

pub trait Process {
    fn write_line(&mut self, msg: &str);
    fn read_line(&mut self) -> String;
    fn is_running(&mut self) -> bool;
    fn lines<'a>(&'a mut self) -> Box<dyn Iterator<Item = String> + 'a>;
    #[allow(dead_code)]
    fn as_any(&self) -> &dyn std::any::Any;
}

pub struct RealProcess {
    proc: Popen,
}

impl RealProcess {
    pub fn new(exec_path: &str) -> Self {
        let proc = Popen::create(
            &[exec_path],
            PopenConfig {
                stdin: Redirection::Pipe,
                stdout: Redirection::Pipe,
                stderr: Redirection::Pipe,
                detached: true,
                ..Default::default()
            },
        )
        .expect("Creating detached stockfish process failed!");

        RealProcess { proc }
    }
}

impl Process for RealProcess {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn write_line(&mut self, msg: &str) {
        if self.proc.stdin.is_none() {
            panic!("stdin is not available");
        }

        if self.proc.poll().is_none() {
            if let Some(stdin) = &mut self.proc.stdin {
                println!("sent {msg}");
                writeln!(stdin, "{}", msg).expect("Failed to write to stdin");
                stdin.flush().expect("Failed to flush stdin");
            }
        }
    }

    fn read_line(&mut self) -> String {
        if let Some(stdout) = &mut self.proc.stdout {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => panic!("EOF reached unexpectedly"),
                Ok(_) => line.trim().to_string(),
                Err(e) => panic!("Error reading stdout: {}", e),
            }
        } else {
            panic!("stdout is not available");
        }
    }

    fn lines<'a>(&'a mut self) -> Box<dyn Iterator<Item = String> + 'a> {
        if let Some(stdout) = &mut self.proc.stdout {
            let reader = BufReader::new(stdout);
            Box::new(reader.lines().filter_map(|l| l.ok()))
        } else {
            Box::new(std::iter::empty())
        }
    }

    fn is_running(&mut self) -> bool {
        self.proc.poll().is_none()
    }
}

/// ```
/// let mut stock = Stockfish::new("/home/leghart/projects/chessify_utils/stockfish_15.1_linux_x64/stockfish-ubuntu-20.04-x86-64");
/// stock.set_config();
/// stock.set_elo_rating(2200);
/// for _ in 0..25 {
///     let t = std::time::Instant::now();
///     let best_move = stock.get_best_move().unwrap();
///     println!("best move: {best_move} {:?}", t.elapsed());
///     stock.make_move(vec![best_move]);
/// }
/// ```
pub struct Stockfish {
    proc: Box<dyn Process>,
    pub parameters: HashMap<String, String>,
    depth: u8,
    info: String,
    quit_sent: bool,
    pub version: String,
}

impl Stockfish {
    pub fn new(exec_path: &str) -> Self {
        let real_proc = RealProcess::new(exec_path);
        Self::new_with_process(Box::new(real_proc))
    }

    pub fn new_with_process(proc: Box<dyn Process>) -> Self {
        let mut _self = Stockfish {
            proc,
            parameters: HashMap::new(),
            depth: 5,
            info: String::new(),
            quit_sent: false,
            version: String::new(),
        };

        _self.version = _self.read_line();
        _self._put("uci"); // start engine
        let _ = _self.read_line(); // clear buffer

        _self
    }

    pub fn set_config(&mut self) {
        let default_params: HashMap<&str, &str> = HashMap::from_iter([
            ("Debug Log File", ""),
            // ("Threads", "1"),
            ("Ponder", "false"),
            ("Hash", "16"),
            ("MultiPV", "1"),
            ("Skill Level", "20"),
            ("Move Overhead", "10"),
            ("UCI_Chess960", "false"),
            ("UCI_LimitStrength", "false"),
            ("UCI_Elo", "1700"),
        ]);

        self.update_params(default_params);
    }

    //TODO:
    pub fn get_wdl_stats(&self) -> Option<Vec<String>> {
        Some(Vec::new())
    }

    // TODO: to fix
    // TODO: add tests
    pub fn get_evaluation(&mut self) -> HashMap<String, String> {
        let mut evaluation: HashMap<String, String> = HashMap::new();
        let fen_position = self.get_fen_position();
        let compare = match fen_position.contains('w') {
            true => 1,
            false => -1,
        };
        self._put(&format!("position {fen_position}"));
        self._go();

        loop {
            let raw = self.read_line();
            let text: Vec<&str> = raw.split(" ").collect();
            if text[0] == "info" {
                for n in 0..text.len() - 1 {
                    if text[n] == "score" {
                        *evaluation.get_mut("type").unwrap() = text[n + 1].to_string();
                        *evaluation.get_mut("value").unwrap() =
                            (text[n + 2].parse::<isize>().unwrap() * compare).to_string();
                    }
                }
            } else if text[0] == "bestmove" {
                return evaluation;
            }
        }
    }

    pub fn set_skill_level(&mut self, level: usize) {
        self.update_params(HashMap::from_iter([
            ("UCI_LimitStrength", "false"),
            ("Skill level", &level.to_string()),
        ]));
    }

    // TODO: add tests
    pub fn set_elo_rating(&mut self, rating: usize) {
        self.update_params(HashMap::from_iter([
            ("UCI_LimitStrength", "true"),
            ("UCI_Elo", &rating.to_string()),
        ]));
    }

    pub fn get_best_move(&mut self) -> Option<String> {
        self._go();
        self.get_move_from_proc()
    }

    // TODO: add tests
    pub fn make_move(&mut self, moves: Vec<String>) {
        if moves.len() == 0 {
            return;
        }

        self.prepare_for_new_position(false);
        for _move in moves {
            if !self.is_correct_move(&_move) {
                panic!("TODO");
            }
            let pos = self.get_fen_position();
            self._put(&format!("position fen {pos} moves {_move}"));
        }
    }

    // TODO: add tests
    fn update_params(&mut self, new_param_values_p: HashMap<&str, &str>) {
        let mut new_param_values = new_param_values_p;

        if !self.parameters.is_empty() {
            for key in new_param_values.keys() {
                if !self.parameters.contains_key(*key) {
                    panic!("TODO"); //TODO!
                }
            }
        }

        if (new_param_values.contains_key("Skill Level")
            != new_param_values.contains_key("UCI_Elo"))
            && !new_param_values.contains_key("UCI_LimitStrength")
        {
            if new_param_values.contains_key("Skill Level") {
                new_param_values.insert("UCI_LimitStrength", "false");
            } else if new_param_values.contains_key("UCI_Elo") {
                new_param_values.insert("UCI_LimitStrength", "true");
            }
        }

        if let Some(threads_value) = new_param_values.remove("Threads") {
            // TODO!: check
            let hash_value = new_param_values.remove("Hash");

            new_param_values.insert("Threads", threads_value);
            if let Some(hash_value) = hash_value {
                new_param_values.insert("Hash", hash_value);
            }
        }

        for (name, value) in new_param_values.iter() {
            self._put(&format!("setoption name {name} value {value}"));
            self.parameters
                .entry(name.to_string())
                .and_modify(|e| *e = value.to_string())
                .or_insert_with(|| value.to_string());
            self.is_ready();
        }

        let pos = self.get_fen_position();
        self.set_fen_position(&pos, false);
    }

    fn get_fen_position(&mut self) -> String {
        self._put("d");

        for line in self.proc.lines() {
            let trimmed = line.trim();
            if trimmed.contains("Fen: ") {
                return trimmed[5..].to_string();
            }
        }
        panic!()
    }

    fn set_fen_position(&mut self, fen: &str, token: bool) {
        self.prepare_for_new_position(token);
        self._put(&format!("position fen {fen}"));
    }

    fn prepare_for_new_position(&mut self, send_token: bool) {
        if send_token {
            self._put("ucinewgame");
        }
        self.is_ready();
        self.info = String::new();
    }

    fn get_move_from_proc(&mut self) -> Option<String> {
        let mut last_text = String::new();

        loop {
            let text = self.proc.read_line();

            let splitted: Vec<&str> = text.split_whitespace().collect();
            if splitted.is_empty() {
                continue;
            }

            if splitted[0] == "bestmove" {
                self.info = last_text;
                if splitted.len() > 1 && splitted[1] == "(none)" {
                    return None;
                } else if splitted.len() > 1 {
                    return Some(splitted[1].to_string());
                } else {
                    return None;
                }
            }

            last_text = text;
        }
    }

    fn go_time(&mut self, time: usize) {
        self._put(&format!("go movetime {time}"));
    }

    fn is_ready(&mut self) {
        self._put("isready");

        loop {
            let _out = self.read_line();
            if _out == "readyok" {
                break;
            }
        }
    }

    fn is_correct_move(&mut self, _move: &str) -> bool {
        let old_info = self.info.clone();
        self._put(&format!("go depth 1 searchmoves {_move}"));
        let result = self.get_move_from_proc().is_some();
        self.info = old_info;
        return result;
    }

    fn read_line(&mut self) -> String {
        self.proc.read_line()
    }

    fn _put(&mut self, cmd: &str) {
        if !self.proc.is_running() && !self.quit_sent {
            return;
        }

        self.proc.write_line(cmd);

        if cmd == "quit" {
            self.quit_sent = true;
        }
    }

    fn _go(&mut self) {
        self._put(&format!("go depth {}", self.depth));
    }
}

impl Drop for Stockfish {
    fn drop(&mut self) {
        self._put("quit");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    pub struct MockProcess {
        pub written_lines: Vec<String>,
        pub lines_to_read: VecDeque<String>,
        pub running: bool,
    }

    impl MockProcess {
        pub fn new() -> Self {
            Self {
                written_lines: Vec::new(),
                lines_to_read: VecDeque::new(),
                running: true,
            }
        }

        pub fn push_read_line(&mut self, line: &str) {
            self.lines_to_read.push_back(line.to_string());
        }

        pub fn set_running(&mut self, running: bool) {
            self.running = running;
        }
    }

    impl Process for MockProcess {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn write_line(&mut self, msg: &str) {
            self.written_lines.push(msg.to_string());
        }

        fn read_line(&mut self) -> String {
            self.lines_to_read.pop_front().unwrap_or_default()
        }

        fn is_running(&mut self) -> bool {
            self.running
        }

        fn lines<'a>(&'a mut self) -> Box<dyn Iterator<Item = String> + 'a> {
            let iter = self.lines_to_read.drain(..).collect::<Vec<_>>().into_iter();
            Box::new(iter)
        }
    }

    #[test]
    fn stockfish_new_with_mock_reads_version_and_sends_uci() {
        let mut mock = MockProcess::new();
        mock.push_read_line("Stockfish 17 by Mock");
        mock.push_read_line("readyok");

        let sf = Stockfish::new_with_process(Box::new(mock));
        assert_eq!(sf.version, "Stockfish 17 by Mock");

        let proc = sf.proc.as_any().downcast_ref::<MockProcess>().unwrap();
        assert!(proc.written_lines.contains(&"uci".to_string()));
    }

    #[test]
    fn set_config() {
        panic!("TODO");
    }

    #[test]
    fn get_fen_position_returns_correct_fen() {
        let mut mock = MockProcess::new();
        mock.push_read_line("Stockfish 17 by Mock");
        mock.push_read_line("readyok");
        mock.push_read_line("Fen: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        mock.push_read_line("readyok");

        let mut sf = Stockfish::new_with_process(Box::new(mock));

        let fen = sf.get_fen_position();

        let proc = sf.proc.as_any().downcast_ref::<MockProcess>().unwrap();
        assert!(proc.written_lines.contains(&"d".to_string()));

        assert_eq!(
            fen,
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );
    }

    #[test]
    fn get_best_move_returns_correct_move() {
        let mut mock = MockProcess::new();

        mock.push_read_line("uciok");
        mock.push_read_line("info depth 10 score cp 20");
        mock.push_read_line("bestmove e2e4");

        let mut sf = Stockfish::new_with_process(Box::new(mock));

        let best_move = sf.get_best_move();

        assert_eq!(best_move, Some("e2e4".to_string()));

        let proc = sf.proc.as_any().downcast_ref::<MockProcess>().unwrap();
        assert!(proc
            .written_lines
            .iter()
            .any(|cmd| cmd.starts_with("go depth")));
    }

    #[test]
    fn get_move_from_proc_returns_bestmove() {
        let mut mock = MockProcess::new();
        mock.push_read_line("Stockfish 17 by Mock");
        mock.push_read_line("readyok");
        mock.push_read_line("info depth 10 score cp 13");
        mock.push_read_line("bestmove e2e4");
        mock.push_read_line("readyok");

        let mut sf = Stockfish::new_with_process(Box::new(mock));

        let best_move = sf.get_move_from_proc();

        assert_eq!(best_move, Some("e2e4".to_string()));
        assert_eq!(sf.info, "info depth 10 score cp 13");
    }

    #[test]
    fn is_correct_move_returns_true_for_valid_move() {
        let mut mock = MockProcess::new();

        mock.push_read_line("Stockfish 17 by Mock");
        mock.push_read_line("readyok");
        mock.push_read_line("bestmove e2e4");
        mock.push_read_line("readyok");

        let mut sf = Stockfish::new_with_process(Box::new(mock));

        let result = sf.is_correct_move("e2e4");
        assert!(result);

        let proc = sf.proc.as_any().downcast_ref::<MockProcess>().unwrap();
        assert!(proc
            .written_lines
            .iter()
            .any(|cmd| cmd.contains("go depth 1 searchmoves e2e4")));
    }

    #[test]
    fn is_correct_move_returns_false_for_invalid_move() {
        let mut mock = MockProcess::new();

        mock.push_read_line("Stockfish 17 by Mock");
        mock.push_read_line("readyok");
        mock.push_read_line("bestmove (none)");
        mock.push_read_line("readyok");

        let mut sf = Stockfish::new_with_process(Box::new(mock));

        let result = sf.is_correct_move("a1a1");
        assert!(!result);

        let proc = sf.proc.as_any().downcast_ref::<MockProcess>().unwrap();
        assert!(proc
            .written_lines
            .iter()
            .any(|cmd| cmd.contains("go depth 1 searchmoves a1a1")));
    }

    #[test]
    fn put_cmd_without_active_poll() {
        let mock = MockProcess {
            written_lines: Vec::new(),
            lines_to_read: VecDeque::new(),
            running: false,
        };

        let mut sf = Stockfish {
            proc: Box::new(mock),
            parameters: HashMap::new(),
            depth: 1,
            info: "".to_string(),
            quit_sent: false,
            version: "".to_string(),
        };

        sf._put("abc");

        let proc = sf.proc.as_any().downcast_ref::<MockProcess>().unwrap();
        assert_eq!(proc.written_lines.len(), 0);
    }

    #[test]
    fn put_cmd_with_active_poll() {
        let mock = MockProcess::new();

        let mut sf = Stockfish {
            proc: Box::new(mock),
            parameters: HashMap::new(),
            depth: 1,
            info: "".to_string(),
            quit_sent: false,
            version: "".to_string(),
        };

        sf._put("abc");

        let proc = sf.proc.as_any().downcast_ref::<MockProcess>().unwrap();
        assert!(proc.written_lines.contains(&"abc".to_string()));
    }

    #[test]
    fn put_cmd_quit_process() {
        let mock = MockProcess::new();

        let mut sf = Stockfish {
            proc: Box::new(mock),
            parameters: HashMap::new(),
            depth: 1,
            info: "".to_string(),
            quit_sent: false,
            version: "".to_string(),
        };

        sf._put("quit");

        assert!(sf.quit_sent);
    }

    #[test]
    fn go_depth_send_correct_msg() {
        let mock = MockProcess::new();

        let mut sf = Stockfish {
            proc: Box::new(mock),
            parameters: HashMap::new(),
            depth: 5,
            info: "".to_string(),
            quit_sent: false,
            version: "".to_string(),
        };

        sf._go();

        let proc = sf.proc.as_any().downcast_ref::<MockProcess>().unwrap();
        assert!(proc.written_lines.contains(&"go depth 5".to_string()));
    }

    #[test]
    fn prepare_for_new_with_token() {
        let mut mock = MockProcess::new();
        mock.push_read_line("readyok");

        let mut sf = Stockfish {
            proc: Box::new(mock),
            parameters: HashMap::new(),
            depth: 1,
            info: "Old info".to_string(),
            quit_sent: false,
            version: "".to_string(),
        };

        sf.prepare_for_new_position(true);

        let proc = sf.proc.as_any().downcast_ref::<MockProcess>().unwrap();
        assert!(proc.written_lines.contains(&"ucinewgame".to_string()));
        assert_eq!(sf.info, String::new());
    }

    #[test]
    fn prepare_for_new_without_start_new_game() {
        let mut mock = MockProcess::new();
        mock.push_read_line("readyok");

        let mut sf = Stockfish {
            proc: Box::new(mock),
            parameters: HashMap::new(),
            depth: 1,
            info: "Old info".to_string(),
            quit_sent: false,
            version: "".to_string(),
        };

        sf.prepare_for_new_position(false);

        let proc = sf.proc.as_any().downcast_ref::<MockProcess>().unwrap();
        assert!(!proc.written_lines.contains(&"ucinewgame".to_string()));
        assert_eq!(sf.info, String::new());
    }

    #[test]
    fn set_fen_position_with_token() {
        let mut mock = MockProcess::new();
        mock.push_read_line("readyok");

        let mut sf = Stockfish {
            proc: Box::new(mock),
            parameters: HashMap::new(),
            depth: 1,
            info: "Old info".to_string(),
            quit_sent: false,
            version: "".to_string(),
        };

        sf.set_fen_position("abc/abc/", true);

        let proc = sf.proc.as_any().downcast_ref::<MockProcess>().unwrap();
        assert!(proc.written_lines.contains(&"ucinewgame".to_string()));
        assert!(proc
            .written_lines
            .contains(&"position fen abc/abc/".to_string()));
    }
}
