use chess::ChessMove;
use std::process::Stdio;
use std::str::FromStr;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

pub const MAX_CONCURRENT_STOCKFISH_INSTANCES: u8 = 3;
const STOCKFISH_PATH: &str = "./engine/stockfish-ubuntu-x86-64-avx2";

pub struct StockfishInstance {
    proc: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    moves: String,
    instance_count: Arc<AtomicU8>,
}
impl StockfishInstance {
    //reduces the instance count when dropped
    pub(crate) async fn new(instance_count: Arc<AtomicU8>) -> Option<Self> {
        if let Ok(mut proc) = Command::new(STOCKFISH_PATH)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
        {
            if proc.stdin.is_none() || proc.stdout.is_none() {
                let _ = proc.kill();
                let _ = proc.wait();
                return None;
            }
            let mut stdin = proc.stdin.take().unwrap();
            let stdout = proc.stdout.take().unwrap();

            let res = stdin.write_all(
                r#"
uci
isready
ucinewgame
setoption name threads value 1
setoption name hash value 32

"#.as_ref()
            ).await;

            if res.is_err() || stdin.flush().await.is_err() {
                let _ = proc.kill();
                let _ = proc.wait();
                return None;
            }

            Some(
                Self {
                    proc,
                    stdin,
                    stdout: BufReader::new(stdout),
                    moves: String::new(),
                    instance_count,
                }
            )
        } else {
            None
        }
    }

    pub async fn get_next_move(&mut self, played_move: ChessMove) -> ChessMove {
        println!("=>>>{}", self.moves);

        self.moves.push(' ');
        self.moves.push_str(&*played_move.to_string());
        self.stdin.write_all(format!("position startpos moves{}\n", self.moves).as_bytes()).await.expect("Error while writing");
        self.stdin.flush().await.expect("Error while flushing");

        self.stdin.write_all("go depth 20\n".as_bytes()).await.expect("Error while writing");
        self.stdin.flush().await.expect("Error while flushing");

        let mut line = String::new();
        loop {
            self.stdout.read_line(&mut line).await.expect("Error reading line");
            if line.trim().starts_with("bestmove ") {
                let mut sp = line.split(' ');
                sp.next();
                let mv_str = sp.next().unwrap().trim();
                let mv = ChessMove::from_str(mv_str).unwrap();
                self.moves.push(' ');
                self.moves.push_str(mv_str);

                return mv;
            }
            line.clear();
        }
    }
}

impl Drop for StockfishInstance {
    fn drop(&mut self) {
        let _ = self.proc.kill();
        let _ = self.proc.wait();
        let decrement_result = self.instance_count.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |val| {
            if val > 0
            { Some(val - 1) }
            else { None }
        });
        if decrement_result.is_err() { eprintln!("Stockfish instance count decrement error - already at 0"); }
    }
}