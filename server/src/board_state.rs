use chb_chess::{Move, Board};
use tokio::sync::{broadcast::{self, Sender, Receiver}, Mutex, MutexGuard};

pub struct BoardState {
    board: Mutex<Board>,
    channel: Sender<Move>
}

impl BoardState {
    pub fn init() -> Self {
        let (channel, _) = broadcast::channel(1);
        BoardState {
            board: Mutex::new(Board::default()),
            channel
        }
    }

    pub fn subscribe(&self) -> Receiver<Move> {
        self.channel.subscribe()
    }

    pub async fn board(&self) -> MutexGuard<Board> {
        self.board.lock().await
    }

    pub async fn make(&self, mv: Move) {
        if self.board.lock().await.make(mv).is_ok() {
            let _ = self.channel.send(mv);
        }
    }
}
