use regex::Regex;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use subprocess::{Popen, PopenConfig, Redirection};

pub struct Summary {
    pub eval: String,
    pub best_lines: Vec<String>,
}

pub trait Process: Send {
    fn write_line(&mut self, msg: &str);
    fn read_line(&mut self) -> String;
    fn lines(&mut self, stop_pattern: &Regex) -> Vec<String>;
    fn desired_line(&mut self, stop_pattern: &Regex) -> String;
    fn is_running(&mut self) -> bool;
    #[allow(dead_code)]
    fn as_any(&self) -> &dyn std::any::Any;
}

pub struct RealProcess {
    proc: Popen,
}

impl RealProcess {
    pub fn new(exec_path: &std::path::PathBuf) -> Self {
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
                log::trace!("Stockfish write line: {msg}");
                writeln!(stdin, "{msg}").expect("Failed to write to stdin");
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
                Ok(_) => {
                    let line = line.trim().to_string();
                    log::trace!("[SINGLE] Stockfish read line: {line}");
                    line
                }
                Err(e) => panic!("Error reading stdout: {e}"),
            }
        } else {
            panic!("stdout is not available");
        }
    }

    fn is_running(&mut self) -> bool {
        self.proc.poll().is_none()
    }

    fn lines(&mut self, stop_pattern: &Regex) -> Vec<String> {
        if let Some(stdout) = &mut self.proc.stdout {
            let mut lines = Vec::new();
            for line in BufReader::new(stdout).lines() {
                let line = line.unwrap();
                log::trace!("[MULTI] Stockfish read line: {line}");

                if stop_pattern.is_match(&line) {
                    lines.push(line);
                    break;
                }
                lines.push(line);
            }
            lines
        } else {
            Vec::new()
        }
    }

    fn desired_line(&mut self, stop_pattern: &Regex) -> String {
        self.lines(stop_pattern)
            .last()
            .expect("Desired line not found")
            .to_owned()
    }
}

pub struct Stockfish {
    proc: Box<dyn Process>,
    pub parameters: HashMap<String, String>,
    depth: u8,
    info: String,
    quit_sent: bool,
    pub version: String,
}

#[allow(dead_code)]
impl Stockfish {
    pub fn new(exec_path: &std::path::PathBuf, depth: u8) -> Self {
        let real_proc = RealProcess::new(exec_path);
        Self::new_with_process(Box::new(real_proc), depth)
    }

    pub fn new_with_process(proc: Box<dyn Process>, depth: u8) -> Self {
        let mut _self = Stockfish {
            proc,
            parameters: HashMap::new(),
            depth,
            info: String::new(),
            quit_sent: false,
            version: String::new(),
        };

        _self.version = _self.read_line();
        _self._put("uci");
        let _ = _self.read_line(); // clear buffer

        _self
    }

    pub fn set_config(&mut self, elo: &str, skill: &str, hash: &str, multi_lines: &str) {
        let default_params: HashMap<&str, &str> = HashMap::from_iter([
            ("Debug Log File", ""),
            // ("Threads", "1"),
            ("Ponder", "false"),
            ("Hash", hash),
            ("MultiPV", multi_lines),
            ("Skill Level", skill),
            ("Move Overhead", "10"),
            ("UCI_Chess960", "false"),
            ("UCI_LimitStrength", "true"),
            ("UCI_Elo", elo),
            ("UCI_ShowWDL", "true"),
        ]);

        self.update_params(default_params);
    }

    //TODO: fix
    //TODO: add tests
    pub fn get_wdl_stats(&mut self) -> [usize; 3] {
        let fen_position = self.get_fen_position();
        self._put(&format!("position {fen_position}"));
        self._go();

        let mut result = [0; 3];
        for line in self.proc.lines(&Regex::new("bestmove").unwrap()) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            if parts[0] == "bestmove" {
                break;
            }

            if parts[0] == "info" {
                if let Some(score_index) = parts.iter().position(|&x| x == "wdl") {
                    if score_index + 3 < parts.len() {
                        let score_win = parts[score_index + 1].parse::<usize>().unwrap();
                        let score_draw = parts[score_index + 2].parse::<usize>().unwrap();
                        let score_lose = parts[score_index + 3].parse::<usize>().unwrap();
                        result = [score_win, score_draw, score_lose];
                    }
                }
            }
        }
        result
    }

    pub fn extract_values(
        &mut self,
        data: &[String],
        nth_line: usize,
        color_scalar: f32,
    ) -> (String, Vec<String>) {
        let mut pvs: Option<Vec<String>> = None;
        let mut eval: Option<String> = None;
        for line in data.iter().rev() {
            if Regex::new(&format!(
                r"info depth {}\s+.*multipv {}",
                self.depth, nth_line
            ))
            .unwrap()
            .is_match(line)
            {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(score_index) = parts.iter().position(|&x| x == "score") {
                    if score_index + 2 < parts.len() {
                        let score_type = parts[score_index + 1];
                        let score_value = parts[score_index + 2];

                        match score_type {
                            "cp" => {
                                if let Ok(cp_val) = score_value.parse::<f32>() {
                                    eval = Some((cp_val * color_scalar / 100.0).to_string());
                                }
                            }
                            "mate" => {
                                if let Ok(mate_val) = score_value.parse::<f32>() {
                                    let sign = match color_scalar.signum() {
                                        1.0 => "".to_string(),
                                        -1.0 => "-".to_string(),
                                        _ => unreachable!(),
                                    };
                                    eval = Some(format!("{sign}M{}", mate_val.abs() as usize));
                                }
                            }
                            _ => {}
                        }
                    }
                }

                if let Some(pv_idx) = parts.iter().position(|&x| x == "pv") {
                    pvs = Some(
                        parts[pv_idx + 1..]
                            .iter()
                            .map(|s| s.to_string())
                            .collect::<Vec<_>>(),
                    );
                }
            }
        }

        (
            eval.expect("evaluation is None"),
            pvs.expect("multipv is None"),
        )
    }

    pub fn summary(&mut self, search_lines: usize) -> Vec<Summary> {
        let fen_position = self.get_fen_position(); // to get current side

        let color_scaler = if fen_position.contains('w') {
            1.0
        } else {
            -1.0
        };

        self._put(&format!("position {fen_position}"));
        self._go();

        let lines = self.proc.lines(&Regex::new("bestmove").unwrap());

        let mut output: Vec<Summary> = Vec::with_capacity(search_lines);
        for nth in 1..=search_lines {
            let (eval, best_lines) = self.extract_values(&lines, nth, color_scaler);
            output.push(Summary { eval, best_lines })
        }
        output
    }

    pub fn set_skill_level(&mut self, level: usize) {
        self.update_params(HashMap::from_iter([
            ("UCI_LimitStrength", "false"),
            ("Skill level", &level.to_string()),
        ]));
    }

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

    pub fn make_move(&mut self, moves: Vec<String>) {
        if moves.is_empty() {
            return;
        }

        self.prepare_for_new_position(false);

        for _move in moves {
            if !self.is_correct_move(&_move) {
                let msg = format!(
                    "Move '{_move}' is not a valid move for current position or engine state."
                );
                panic!("{msg}");
            }

            let pos = self.get_fen_position();
            self._put(&format!("position fen {pos} moves {_move}"));
        }
    }

    // TODO: fix threads
    fn update_params(&mut self, new_param_values_p: HashMap<&str, &str>) {
        let mut new_param_values = new_param_values_p;

        if !self.parameters.is_empty() {
            for key in new_param_values.keys() {
                if !self.parameters.contains_key(*key) {
                    panic!("TODO!");
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

        let binding = self.proc.desired_line(&Regex::new("Fen").unwrap());
        let line = binding.trim();
        if line.contains("Fen: ") {
            return line[5..].to_string();
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

        for text in self.proc.lines(&Regex::new("bestmove").unwrap()) {
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
        None
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
        result
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
    }

    impl Process for MockProcess {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn write_line(&mut self, msg: &str) {
            self.written_lines.push(msg.to_string());
        }

        fn read_line(&mut self) -> String {
            self.lines_to_read.pop_front().unwrap()
        }

        fn is_running(&mut self) -> bool {
            self.running
        }

        fn lines(&mut self, stop_pattern: &Regex) -> Vec<String> {
            let mut lines = Vec::new();
            for line in self.lines_to_read.iter().cloned() {
                lines.push(line.clone());
                if stop_pattern.is_match(&line) {
                    break;
                }
            }
            lines
        }

        fn desired_line(&mut self, stop_pattern: &Regex) -> String {
            self.lines(stop_pattern).last().unwrap().to_owned()
        }
    }

    #[test]
    fn stockfish_new_with_mock_reads_version_and_sends_uci() {
        let mut mock = MockProcess::new();
        mock.push_read_line("Stockfish 17 by Mock");
        mock.push_read_line("readyok");

        let sf = Stockfish::new_with_process(Box::new(mock), 1);
        assert_eq!(sf.version, "Stockfish 17 by Mock");

        let proc = sf.proc.as_any().downcast_ref::<MockProcess>().unwrap();
        assert!(proc.written_lines.contains(&"uci".to_string()));
    }

    #[test]
    fn get_fen_position_returns_correct_fen() {
        let mut mock = MockProcess::new();
        mock.push_read_line("Stockfish 17 by Mock");
        mock.push_read_line("readyok");
        mock.push_read_line("Fen: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        mock.push_read_line("readyok");

        let mut sf = Stockfish::new_with_process(Box::new(mock), 1);

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

        let mut sf = Stockfish::new_with_process(Box::new(mock), 1);

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

        let mut sf = Stockfish::new_with_process(Box::new(mock), 1);

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

        let mut sf = Stockfish::new_with_process(Box::new(mock), 1);

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

        let mut sf = Stockfish::new_with_process(Box::new(mock), 1);

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

    #[test]
    fn set_elo_rating_updates_parameters() {
        let mut mock = MockProcess::new();
        mock.push_read_line("Stockfish 17 by Mock");
        mock.push_read_line("readyok");

        mock.push_read_line("readyok"); // UCI_LimitStrength
        mock.push_read_line("readyok"); // UCI_Elo

        mock.push_read_line("Fen: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        mock.push_read_line("readyok");

        let mut sf = Stockfish::new_with_process(Box::new(mock), 1);
        sf.set_elo_rating(2000);

        let proc = sf.proc.as_any().downcast_ref::<MockProcess>().unwrap();
        assert!(proc
            .written_lines
            .contains(&"setoption name UCI_LimitStrength value true".to_string()));
        assert!(proc
            .written_lines
            .contains(&"setoption name UCI_Elo value 2000".to_string()));
        assert_eq!(
            sf.parameters.get("UCI_LimitStrength"),
            Some(&"true".to_string())
        );
        assert_eq!(sf.parameters.get("UCI_Elo"), Some(&"2000".to_string()));
    }

    #[test]
    fn update_params_sends_correct_commands() {
        let mut mock = MockProcess::new();
        mock.push_read_line("Stockfish 17 by Mock");
        mock.push_read_line("readyok");

        mock.push_read_line("readyok"); // MyParam
        mock.push_read_line("readyok"); // Threads
        mock.push_read_line("readyok"); // Hash

        mock.push_read_line("Fen: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        mock.push_read_line("readyok");

        let mut sf = Stockfish::new_with_process(Box::new(mock), 1);
        let mut params = HashMap::new();
        params.insert("MyParam", "Value1");
        params.insert("Threads", "4");
        params.insert("Hash", "256");

        sf.update_params(params);

        let proc = sf.proc.as_any().downcast_ref::<MockProcess>().unwrap();
        assert!(proc
            .written_lines
            .contains(&"setoption name MyParam value Value1".to_string()));
        assert!(proc
            .written_lines
            .contains(&"setoption name Threads value 4".to_string()));
        assert!(proc
            .written_lines
            .contains(&"setoption name Hash value 256".to_string()));

        assert_eq!(sf.parameters.get("MyParam"), Some(&"Value1".to_string()));
        assert_eq!(sf.parameters.get("Threads"), Some(&"4".to_string()));
        assert_eq!(sf.parameters.get("Hash"), Some(&"256".to_string()));
    }

    #[test]
    fn make_move_sends_correct_commands() {
        let mut mock = MockProcess::new();
        mock.push_read_line("Stockfish 17 by Mock");
        mock.push_read_line("readyok");

        mock.push_read_line("readyok");

        mock.push_read_line("Fen: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        mock.push_read_line("readyok");

        mock.push_read_line("bestmove d2d1");

        mock.push_read_line("info depth 1 bestmove e2e4");
        mock.push_read_line("readyok");
        mock.push_read_line("Fen: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        mock.push_read_line("readyok");

        let mut sf = Stockfish::new_with_process(Box::new(mock), 1);
        sf.make_move(vec!["e2e4".to_string()]);

        let proc = sf.proc.as_any().downcast_ref::<MockProcess>().unwrap();
        assert!(proc.written_lines.contains(
            &"position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 moves e2e4"
                .to_string()
        ));
    }

    #[test]
    #[should_panic(
        expected = "Move 'd1d1' is not a valid move for current position or engine state."
    )]
    fn make_move_panics_on_invalid_move() {
        let mut mock = MockProcess::new();
        mock.push_read_line("Stockfish 17 by Mock");
        mock.push_read_line("readyok");

        mock.push_read_line("readyok");

        mock.push_read_line("Fen: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        mock.push_read_line("readyok");

        mock.push_read_line("bestmove (none)");

        mock.push_read_line("info depth 1 bestmove (none)");
        mock.push_read_line("readyok");
        mock.push_read_line("Fen: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        mock.push_read_line("readyok");

        let mut sf = Stockfish::new_with_process(Box::new(mock), 1);
        sf.make_move(vec!["d1d1".to_string()]);
    }

    #[test]
    fn extract_value_returns_cp_score_for_white() {
        let mut mock = MockProcess::new();

        mock.push_read_line("Stockfish 17 by Mock");
        mock.push_read_line("readyok");

        let data = [
            "info depth 10 selmove 20 multipv 1 score cp 37 nodes 12345 pv e1e2 a2b4".to_string(),
            "info depth 11 selmove 40 multipv 1 score cp 42 nodes 13000 pv d1d2 c1c2".to_string(),
            "bestmove e2e4".to_string(),
        ];
        let mut sf = Stockfish::new_with_process(Box::new(mock), 11);

        let (eval, pv) = sf.extract_values(&data, 1, 1.0);

        assert_eq!(eval, "0.42".to_string());
        assert_eq!(pv, vec!["d1d2".to_string(), "c1c2".to_string()]);
    }

    #[test]
    fn extract_value_returns_cp_score_for_black() {
        let mut mock = MockProcess::new();

        mock.push_read_line("Stockfish 17 by Mock");
        mock.push_read_line("readyok");

        let data = [
            "info depth 10 selmove 20 multipv 1 score cp 37 nodes 12345 pv e1e2 a2b4".to_string(),
            "info depth 11 selmove 40 multipv 1 score cp 47 nodes 13000 pv d1d2".to_string(),
            "bestmove e2e4".to_string(),
        ];
        let mut sf = Stockfish::new_with_process(Box::new(mock), 11);

        let (eval, pv) = sf.extract_values(&data, 1, -1.0);

        assert_eq!(eval, "-0.47".to_string());
        assert_eq!(pv, vec!["d1d2".to_string()]);
    }

    #[test]
    fn extract_value_returns_mate_for_white() {
        let mut mock = MockProcess::new();

        mock.push_read_line("Stockfish 17 by Mock");
        mock.push_read_line("readyok");

        let data = [
            "info depth 10 selmove 20 multipv 1 score cp 37 nodes 12345 pv e1e2 a2b4".to_string(),
            "info depth 11 selmove 40 multipv 1 score mate 2 nodes 13000 pv d1d2".to_string(),
            "bestmove e2e4".to_string(),
        ];
        let mut sf = Stockfish::new_with_process(Box::new(mock), 11);

        let (eval, pv) = sf.extract_values(&data, 1, 1.0);

        assert_eq!(eval, "M2".to_string());
        assert_eq!(pv, vec!["d1d2".to_string()]);
    }

    #[test]
    fn extract_value_returns_mate_for_black() {
        let mut mock = MockProcess::new();

        mock.push_read_line("Stockfish 17 by Mock");
        mock.push_read_line("readyok");

        let data = [
            "info depth 10 selmove 20 multipv 1 score cp 37 nodes 12345 pv e1e2 a2b4".to_string(),
            "info depth 11 selmove 40 multipv 1 score mate 1 nodes 13000 pv d1d2".to_string(),
            "bestmove e2e4".to_string(),
        ];
        let mut sf = Stockfish::new_with_process(Box::new(mock), 11);

        let (eval, pv) = sf.extract_values(&data, 1, -1.0);

        assert_eq!(eval, "-M1".to_string());
        assert_eq!(pv, vec!["d1d2".to_string()]);
    }

    #[ignore = "to fix"]
    #[test]
    fn get_evaluation_returns_zero_if_no_score() {
        // let mut mock = MockProcess::new();

        // mock.push_read_line("Stockfish 17 by Mock");
        // mock.push_read_line("readyok");

        // mock.push_read_line("Fen: 8/8/8/8/8/8/K1k5/8 w - - 0 1");

        // mock.push_read_line("bestmove a1a1");

        // let mut sf = Stockfish::new_with_process(Box::new(mock), 1);

        // let eval = sf.get_evaluation();

        // assert_eq!(eval, "0".to_string());
    }
}
