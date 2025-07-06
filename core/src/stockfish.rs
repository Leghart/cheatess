use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use subprocess::{Popen, PopenConfig, Redirection};

/// ```
/// let mut stock = Stockfish::new("/home/leghart/projects/chessify_utils/stockfish_15.1_linux_x64/stockfish-ubuntu-20.04-x86-64");
/// stock.set_elo_rating(2200);
/// for _ in 0..25 {
///     let t = std::time::Instant::now();
///     let best_move = stock.get_best_move().unwrap();
///     println!("best move: {best_move} {:?}", t.elapsed());
///     stock.make_move(vec![best_move]);
/// }
/// ```
pub struct Stockfish {
    proc: Popen,
    pub parameters: HashMap<String, String>,
    depth: u8,
    info: String,
    quit_sent: bool,
    pub version: String,
}

impl Stockfish {
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

        let mut _self = Stockfish {
            proc,
            parameters: HashMap::new(),
            depth: 5,
            info: String::new(),
            quit_sent: false,
            version: String::new(),
        };

        _self.version = _self.read_line();
        _self._put("uci");
        let _ = _self.read_line(); // clear buffer

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
            ("UCI_Elo", "1350"),
        ]);

        _self.update_params(default_params);

        _self
    }

    //TODO:
    pub fn get_wdl_stats(&self) -> Option<Vec<String>> {
        Some(Vec::new())
    }

    // TODO: to fix
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

    pub fn set_elo_rating(&mut self, rating: usize) {
        self.update_params(HashMap::from_iter([
            ("UCI_LimitStrength", "true"),
            ("UCI_Elo", &rating.to_string()),
        ]));
    }

    // TODO?: add wtime & btime
    pub fn get_best_move(&mut self) -> Option<String> {
        self._go();
        self.get_move_from_proc()
    }

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

    // TODO: terrible (think how to solve blocking bufread)
    pub fn get_fen_position(&mut self) -> String {
        self._put("d");
        if let Some(stdout) = self.proc.stdout.as_mut() {
            let reader = BufReader::new(stdout);

            for line_result in reader.lines() {
                match line_result {
                    Ok(line) => {
                        let trimmed = line.trim().to_string();
                        if trimmed.contains("Fen: ") {
                            return trimmed[5..].to_string();
                        }
                    }
                    Err(_) => {
                        panic!("TODO");
                    }
                }
            }
            panic!("TODO");
        } else {
            panic!("TODO");
        }
    }

    fn _put(&mut self, cmd: &str) {
        if self.proc.stdin.is_none() {
            panic!("TODO");
        }
        if self.proc.poll().is_none() && !self.quit_sent {
            if let Some(stdin) = &mut self.proc.stdin {
                // println!("{}", format!("send cmd: {cmd}"));
                writeln!(stdin, "{cmd}").unwrap();
                stdin.flush().unwrap();
            }

            if cmd == "quit" {
                self.quit_sent = true;
            }
        }
    }

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
            // .or_else(|| self.parameters.get("Hash").map(|x| x.as_str()));

            new_param_values.insert("Threads", threads_value);
            if let Some(hash_value) = hash_value {
                new_param_values.insert("Hash", hash_value);
            }
        }

        for (name, value) in new_param_values.iter() {
            self.set_option(name, value, true);
        }

        let pos = self.get_fen_position();
        self.set_fen_position(&pos, false);
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

    fn set_option(&mut self, name: &str, value: &str, update_attr: bool) {
        self._put(&format!("setoption name {name} value {value}"));
        if update_attr {
            self.parameters
                .entry(name.to_string())
                .and_modify(|e| *e = value.to_string())
                .or_insert_with(|| value.to_string());
        }
        self.is_ready();
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
        if let Some(stdout) = self.proc.stdout.as_mut() {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => panic!("TODO"),
                Ok(_) => line.trim().to_string(),
                Err(_) => {
                    panic!("TODO")
                }
            }
        } else {
            panic!("TODO")
        }
    }

    fn _go(&mut self) {
        self._put(&format!("go depth {}", self.depth));
    }

    fn go_time(&mut self, time: usize) {
        self._put(&format!("go movetime {time}"));
    }

    // TODO: terrbile
    fn get_move_from_proc(&mut self) -> Option<String> {
        let mut last_text = String::from("");
        if let Some(stdout) = self.proc.stdout.as_mut() {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                match line {
                    Ok(text) => {
                        let splitted: Vec<&str> = text.split(" ").collect();
                        if splitted[0] == "bestmove" {
                            self.info = last_text;
                            if splitted[1] == "(none)" {
                                return None;
                            } else {
                                return Some(splitted[1].to_string());
                            }
                        }
                        last_text = text;
                    }
                    Err(_) => panic!("TODO"),
                }
            }
            panic!("TODO")
        } else {
            panic!("TODO")
        }
    }
}

impl Drop for Stockfish {
    fn drop(&mut self) {
        self._put("quit");
    }
}
