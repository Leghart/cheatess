use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use subprocess::{Popen, PopenConfig, Redirection};

pub struct Stockfish {
    proc: Popen,
    parameters: HashMap<String, String>,
    depth: u8,
    info: String,
    quit_sent: bool,
    version: String,
}

impl Stockfish {
    pub fn new(exec_path: &str) -> Self {
        let mut proc = Popen::create(
            &[exec_path],
            PopenConfig {
                stdin: Redirection::Pipe,
                stdout: Redirection::Pipe,
                stderr: Redirection::Pipe,
                detached: true,
                ..Default::default()
            },
        )
        .expect("error");

        Stockfish {
            proc,
            parameters: HashMap::new(),
            depth: 15,
            info: String::new(),
            quit_sent: false,
            version: String::new(),
        }
    }

    pub fn init(&mut self, depth: Option<u8>) {
        self.version = self.read_line();

        if let Some(d) = depth {
            self.depth = d;
        }

        self.put("uci");
        let tmp = self.read_line();
        println!("uci: {}", tmp);

        let default_params: HashMap<String, String> = HashMap::from_iter([
            ("Debug Log File".to_string(), "".to_string()),
            // ("Contempt".to_string(), "0".to_string()),
            // ("Min Split Depth".to_string(), "0".to_string()),
            ("Threads".to_string(), "1".to_string()),
            ("Ponder".to_string(), "false".to_string()),
            ("Hash".to_string(), "16".to_string()),
            ("MultiPV".to_string(), "1".to_string()),
            ("Skill Level".to_string(), "20".to_string()),
            ("Move Overhead".to_string(), "10".to_string()),
            // ("Minimum Thinking Time".to_string(), "20".to_string()),
            ("Slow Mover".to_string(), "100".to_string()),
            ("UCI_Chess960".to_string(), "false".to_string()),
            ("UCI_LimitStrength".to_string(), "false".to_string()),
            ("UCI_Elo".to_string(), "1350".to_string()),
        ]);

        self.update_params(default_params);
    }

    fn put(&mut self, cmd: &str) {
        println!("Sending {cmd}");
        if self.proc.stdin.is_none() {
            panic!("TODO");
        }
        if self.proc.poll().is_none() && !self.quit_sent {
            if let Some(stdin) = &mut self.proc.stdin {
                writeln!(stdin, "{}", cmd).unwrap();
                stdin.flush().unwrap();
            }

            if cmd == "quit" {
                self.quit_sent = true;
            }
        }
    }

    fn update_params(&mut self, new_param_values_p: HashMap<String, String>) {
        let mut new_param_values = new_param_values_p;

        if !self.parameters.is_empty() {
            for key in new_param_values.keys() {
                if !self.parameters.contains_key(key) {
                    panic!("Key not exists"); //TODO!
                }
            }
        }

        if (new_param_values.contains_key("Skill Level")
            != new_param_values.contains_key("UCI_Elo"))
            && !new_param_values.contains_key("UCI_LimitStrength")
        {
            if new_param_values.contains_key("Skill Level") {
                new_param_values.insert("UCI_LimitStrength".to_string(), "false".to_string());
            } else if new_param_values.contains_key("UCI_Elo") {
                new_param_values.insert("UCI_LimitStrength".to_string(), "true".to_string());
            }
        }

        if let Some(threads_value) = new_param_values.remove("Threads") {
            let hash_value = new_param_values
                .remove("Hash")
                .or_else(|| self.parameters.get("Hash").cloned());

            new_param_values.insert("Threads".to_string(), threads_value);
            if let Some(hash_value) = hash_value {
                new_param_values.insert("Hash".to_string(), hash_value);
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
        self.put(&format!("position fen {fen}"));
    }

    pub fn prepare_for_new_position(&mut self, send_token: bool) {
        if send_token {
            self.put("ucinewgame");
        }
        self.is_ready();
        self.info = String::new();
    }

    fn set_option(&mut self, name: &str, value: &str, update_attr: bool) {
        self.put(&format!("setoption name {name} value {value}"));

        if update_attr {
            self.parameters
                .entry(name.to_string())
                .and_modify(|e| *e = value.to_string())
                .or_insert_with(|| value.to_string());
        }
        self.is_ready();
    }

    pub fn is_ready(&mut self) {
        self.put("isready");
        let out = self.read_line();
        while out != "readyok" {
            continue;
        }
    }

    pub fn read_line(&mut self) -> String {
        if let Some(stdout) = self.proc.stdout.as_mut() {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => panic!(),
                Ok(_) => {
                    let tmp = line.trim().to_string();
                    println!("<<: {}", tmp);
                    tmp
                }
                Err(e) => {
                    panic!()
                }
            }
        } else {
            panic!()
        }
    }

    // TODO: terrible (think how to solve blocking bufread)
    pub fn get_fen_position(&mut self) -> String {
        self.put("d");
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
                    Err(e) => {
                        eprintln!("Error reading line: {}", e);
                        panic!();
                    }
                }
            }
            panic!();
        } else {
            panic!();
        }
    }

    fn go(&mut self) {
        self.put(&format!("go depth {}", self.depth));
    }

    fn go_time(&mut self, time: usize) {
        self.put(&format!("go movetime {time}"));
    }

    pub fn set_skill_level(&mut self, level: usize) {
        self.update_params(HashMap::from_iter([
            ("UCI_LimitStrength".to_string(), "false".to_string()),
            ("Skill level".to_string(), level.to_string()),
        ]));
    }

    pub fn set_elo_rating(&mut self, rating: usize) {
        self.update_params(HashMap::from_iter([
            ("UCI_LimitStrength".to_string(), "true".to_string()),
            ("UCI_Elo".to_string(), rating.to_string()),
        ]));
    }

    // TODO: add wtime & btime
    pub fn get_best_move(&mut self) -> Option<String> {
        self.go();
        let best_move = self.get_move_from_proc();
        best_move
    }

    fn get_move_from_proc(&mut self) -> Option<String> {
        let mut last_text = String::from("");
        while true {
            let text = self.read_line();
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
        None
    }

    // TODO
    pub fn get_wdl_stats(&self) -> Option<Vec<String>> {
        Some(Vec::new())
    }

    // pub fn get_fen_position(&mut self) -> String {
    // self.put("d");
    // loop {
    // let text = self.read_line();
    // let text: Vec<&str> = text.split(" ").collect();
    // println!("splited: {:?}", text);
    // if text[0] == "Fen:" {
    // while !self.read_line().contains("Checkers") {
    // continue;
    // }
    // return text[1..].join(" ");
    // }
    // }
    // }

    pub fn get_evaluation(&mut self) -> HashMap<String, String> {
        let mut evaluation: HashMap<String, String> = HashMap::new();
        let fen_position = self.get_fen_position();
        let compare = match fen_position.contains('w') {
            true => 1,
            false => -1,
        };
        self.put(&format!("position {fen_position}"));
        self.go();

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

    pub fn get_parameters(&self) -> &HashMap<String, String> {
        &self.parameters
    }
}
