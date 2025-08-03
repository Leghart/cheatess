#![cfg(feature = "async")]

use std::collections::HashMap;
use std::path::PathBuf;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;

#[async_trait::async_trait]
pub trait AsyncProcess: Send + Sync {
    async fn write_line(&mut self, msg: &str);
    async fn read_line(&mut self) -> String;
    async fn lines(&mut self, stop: &str) -> Vec<String>;
    async fn is_running(&mut self) -> bool;
    #[allow(dead_code)]
    fn as_any(&self) -> &dyn std::any::Any;
}

pub struct RealAsyncProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: AsyncBufReader<ChildStdout>,
}

impl RealAsyncProcess {
    pub async fn new(exec_path: &PathBuf) -> Self {
        let mut child = Command::new(exec_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("Creating stockfish process failed!");

        let stdin = child.stdin.take().expect("Failed to open stdin");
        let stdout = child.stdout.take().expect("Failed to open stdout");
        let stdout = AsyncBufReader::new(stdout);

        RealAsyncProcess {
            child,
            stdin,
            stdout,
        }
    }
}

#[async_trait::async_trait]
impl AsyncProcess for RealAsyncProcess {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn write_line(&mut self, msg: &str) {
        log::trace!("write line to stockfish: {msg}");
        self.stdin
            .write_all(format!("{msg}\n").as_bytes())
            .await
            .expect("Failed to write to stdin");

        self.stdin.flush().await.expect("Failed to flush stdin");
    }

    async fn read_line(&mut self) -> String {
        let mut line: String = String::new();
        self.stdout
            .read_line(&mut line)
            .await
            .expect("Error reading stdout");

        let line = line.trim().to_string();
        log::trace!("read line from stockfish: {line}");
        line
    }

    async fn lines(&mut self, stop: &str) -> Vec<String> {
        let mut lines = Vec::new();

        loop {
            let line = self.read_line().await;
            if line.starts_with(stop) {
                lines.push(line);
                break;
            }
            lines.push(line);
        }
        lines
    }

    async fn is_running(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(None) => true,
            _ => false,
        }
    }
}

pub struct AsyncStockfish {
    proc: Mutex<Box<dyn AsyncProcess>>,
    pub parameters: Mutex<HashMap<String, String>>,
    depth: u8,
    info: Mutex<String>,
    quit_sent: Mutex<bool>,
    pub version: Mutex<String>,
}

impl AsyncStockfish {
    pub async fn new(exec_path: &PathBuf, depth: u8) -> Self {
        let proc = RealAsyncProcess::new(exec_path).await;
        Self::new_with_process(Box::new(proc), depth).await
    }

    pub async fn new_with_process(proc: Box<dyn AsyncProcess>, depth: u8) -> Self {
        let instance = AsyncStockfish {
            proc: Mutex::new(proc),
            parameters: Mutex::new(HashMap::new()),
            depth,
            info: Mutex::new(String::new()),
            quit_sent: Mutex::new(false),
            version: Mutex::new(String::new()),
        };

        let version = instance.read_line().await;
        *instance.version.lock().await = version;

        instance._put("uci").await;
        let _ = instance.read_line().await;

        instance
    }

    pub async fn set_config(&self, elo: &str, skill: &str, hash: &str) {
        let default_params: HashMap<&str, &str> = HashMap::from_iter([
            ("Debug Log File", ""),
            ("Ponder", "false"),
            ("Hash", hash),
            ("MultiPV", "1"),
            ("Skill Level", skill),
            ("Move Overhead", "10"),
            ("UCI_Chess960", "false"),
            ("UCI_LimitStrength", "true"),
            ("UCI_Elo", elo),
            ("UCI_ShowWDL", "true"),
        ]);
        self.update_params(default_params).await;
    }

    pub async fn get_evaluation(&self) -> String {
        let fen_position = self.get_fen_position().await;
        let compare = if fen_position.contains('w') {
            1.0
        } else {
            -1.0
        };

        self._put(&format!("position {fen_position}")).await;
        self._go().await;

        let mut evaluation_cp: Option<f32> = None;
        let mut evaluation_mate: Option<f32> = None;

        let lines = self.proc.lock().await.lines("bestmove").await;
        for line in lines {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }
            if parts[0] == "bestmove" {
                break;
            }
            if parts[0] == "info" {
                if let Some(score_index) = parts.iter().position(|&x| x == "score") {
                    if score_index + 2 < parts.len() {
                        let score_type = parts[score_index + 1];
                        let score_value = parts[score_index + 2];
                        match score_type {
                            "cp" => {
                                if let Ok(cp_val) = score_value.parse::<f32>() {
                                    evaluation_cp = Some(cp_val * compare);
                                }
                            }
                            "mate" => {
                                if let Ok(mate_val) = score_value.parse::<f32>() {
                                    evaluation_mate = Some(mate_val);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        if let Some(mate) = evaluation_mate {
            let sign = match mate.signum() {
                1.0 => "".to_string(),
                -1.0 => "-".to_string(),
                _ => unreachable!(),
            };
            let abs = mate.abs() as usize;
            return format!("{sign}M{abs}");
        }

        (evaluation_cp.unwrap_or(0.0) / 100.0).to_string()
    }

    #[allow(dead_code)]
    pub async fn set_skill_level(&self, level: usize) {
        self.update_params(HashMap::from_iter([
            ("UCI_LimitStrength", "false"),
            ("Skill level", &level.to_string()),
        ]))
        .await;
    }

    #[allow(dead_code)]
    pub async fn set_elo_rating(&self, rating: usize) {
        self.update_params(HashMap::from_iter([
            ("UCI_LimitStrength", "true"),
            ("UCI_Elo", &rating.to_string()),
        ]))
        .await;
    }

    pub async fn get_best_move(&self) -> Option<String> {
        self._go().await;
        self.get_move_from_proc().await
    }

    pub async fn make_move(&self, moves: Vec<String>) {
        if moves.is_empty() {
            return;
        }

        self.prepare_for_new_position(false).await;

        for mv in moves {
            if !self.is_correct_move(&mv).await {
                panic!("Move '{mv}' is not a valid move for current position or engine state.");
            }
            let pos = self.get_fen_position().await;
            self._put(&format!("position fen {pos} moves {mv}")).await;
        }
    }

    async fn update_params(&self, new_param_values_p: HashMap<&str, &str>) {
        let mut new_param_values = new_param_values_p;

        let mut parameters = self.parameters.lock().await;

        if !parameters.is_empty() {
            for key in new_param_values.keys() {
                if !parameters.contains_key(*key) {
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
            if let Some(hash_val) = hash_value {
                new_param_values.insert("Hash", hash_val);
            }
        }

        for (k, v) in &new_param_values {
            parameters.insert(k.to_string(), v.to_string());
        }

        let mut proc = self.proc.lock().await;

        for (param_name, param_value) in &new_param_values {
            proc.write_line(&format!("setoption name {param_name} value {param_value}"))
                .await;
        }
    }

    async fn prepare_for_new_position(&self, ignore_moves: bool) {
        if ignore_moves {
            self._put("ucinewgame").await;
        }
        self.is_ready().await;
        let mut info = self.info.lock().await;
        *info = String::new();
    }

    async fn is_ready(&self) {
        self._put("isready").await;
        let mut proc = self.proc.lock().await;
        while proc.read_line().await != "readyok" {}
    }

    async fn _go(&self) {
        self._put(&format!("go depth {}", self.depth)).await;
    }

    async fn _put(&self, s: &str) {
        let mut proc = self.proc.lock().await;

        if !proc.is_running().await && !*self.quit_sent.lock().await {
            return;
        }

        proc.write_line(s).await;
    }

    async fn read_line(&self) -> String {
        let mut proc = self.proc.lock().await;
        proc.read_line().await
    }

    async fn get_move_from_proc(&self) -> Option<String> {
        let mut last_text = String::new();
        let mut info = self.info.lock().await;
        let lines = self.proc.lock().await.lines("bestmove").await;

        for line in lines {
            let splitted: Vec<&str> = line.split_whitespace().collect();
            if splitted.is_empty() {
                continue;
            }

            if splitted[0] == "bestmove" {
                *info = last_text;
                if splitted.len() > 1 && splitted[1] == "(none)" {
                    return None;
                } else if splitted.len() > 1 {
                    return Some(splitted[1].to_string());
                } else {
                    return None;
                }
            }

            last_text = line;
        }
        None
    }

    async fn get_fen_position(&self) -> String {
        self._put("d").await;
        let lines = self.proc.lock().await.lines("Fen").await;

        for line in lines {
            let trimmed = line.trim();
            if trimmed.contains("Fen: ") {
                return trimmed[5..].to_string();
            }
        }
        panic!();
    }

    async fn is_correct_move(&self, _move: &str) -> bool {
        let old_info: String = self.info.lock().await.clone();
        self._put(&format!("go depth 1 searchmoves {_move}")).await;
        let result = self.get_move_from_proc().await.is_some();
        let mut info = self.info.lock().await;
        *info = old_info;
        result
    }
}
