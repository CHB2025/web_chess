use chb_chess::{Board, Move};
use tokio::sync::{
    broadcast::{self, Receiver, Sender},
    RwLock, RwLockReadGuard,
};

pub struct BoardState {
    board: RwLock<Board>,
    channel: Sender<Move>,
}

impl BoardState {
    pub fn init() -> Self {
        let (channel, _) = broadcast::channel(1);
        BoardState {
            board: RwLock::new(Board::default()),
            channel,
        }
    }

    pub fn subscribe(&self) -> Receiver<Move> {
        self.channel.subscribe()
    }

    pub async fn board(&self) -> RwLockReadGuard<Board> {
        self.board.read().await
    }

    pub async fn fen(&self) -> String {
        self.board().await.to_fen()
    }

    pub async fn make(&self, mv: Move) {
        if self.board.write().await.make(mv).is_ok() {
            let _ = self.channel.send(mv);
        }
    }
}
