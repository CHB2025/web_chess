use std::sync::Arc;

use anyhow::{anyhow, Result};
use chb_chess::{Board, Color, Move};
use tokio::{
    sync::{
        broadcast::{self, Receiver, Sender},
        Mutex,
    },
    task,
};

use crate::participant::Participant;

pub type Player = Arc<Mutex<dyn Participant + Send + 'static>>;

pub struct Game {
    board: Board,
    game_state: GameState,
    broadcast: Sender<Move>,
}

#[derive(Clone)]
pub enum GameState {
    Active([Player; 2]),
    Setup([Option<Player>; 2]),
}

impl Game {
    pub fn new(board: Board) -> Game {
        let (broadcast, _) = broadcast::channel(1);
        Game {
            board,
            broadcast,
            game_state: GameState::Setup([None, None]),
        }
    }

    pub async fn do_turn(&mut self) -> Result<()> {
        if !self.is_active() {
            return Err(anyhow!("Cannot do turn in inactive state"));
        }
        let mut mv = self.get_next_move().await?;
        while self.board.make(mv).is_err() {
            mv = self.get_next_move().await?;
        }
        self.broadcast.send(mv)?;
        let GameState::Active(players) = &self.game_state else {
            unreachable!("Game is active");
        };
        for color in [Color::White, Color::Black] {
            if players[color].lock().await.send_move(mv).await.is_err() {
                self.set_player(Color::White, None);
                break;
            }
        }
        Ok(())
    }

    pub async fn get_next_move(&self) -> Result<Move> {
        match &self.game_state {
            GameState::Active(players) => {
                players[self.board.color_to_move()]
                    .lock()
                    .await
                    .get_move()
                    .await
            }
            GameState::Setup(_) => Err(anyhow!("Cannot get move for inactive game")),
        }
    }

    pub fn watch(&self) -> Receiver<Move> {
        self.broadcast.subscribe()
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn fen(&self) -> String {
        self.board.to_fen()
    }

    pub fn set_player(&mut self, color: Color, player: Option<Player>) {
        match (&mut self.game_state, player) {
            (GameState::Setup(players), p @ _) => players[color] = p,
            (GameState::Active(players), Some(p)) => players[color] = p,
            (GameState::Active(_), None) => {
                self.into_setup();
                self.set_player(color, None);
            }
        }
        _ = self.into_active()
    }

    pub fn is_active(&self) -> bool {
        matches!(self.game_state, GameState::Active(_))
    }

    fn into_active(&mut self) -> Result<()> {
        let GameState::Setup(players) = self.game_state.clone() else {
            return Ok(());
        };
        match players {
            [Some(white), Some(black)] => {
                self.game_state = GameState::Active([white, black]);
                // notify players/spectators that game is starting
                Ok(())
            }
            _ => Err(anyhow!("Players missing!")),
        }
    }

    fn into_setup(&mut self) {
        match self.game_state.clone() {
            GameState::Active([white, black]) => {
                // notify players/spectators that game is pausing
                self.game_state = GameState::Setup([Some(white), Some(black)])
            }
            GameState::Setup(_) => (),
        }
    }
}

pub trait ExecExt {
    fn start(self);
}

impl ExecExt for Arc<Mutex<Game>> {
    fn start(self) {
        task::spawn(async move {
            loop {
                let mut g = self.lock().await;
                if g.is_active() {
                    if g.do_turn().await.is_err() {}
                } else {
                    break;
                }
                drop(g);
                // need to yield time when not holding a mutable reference to the game
                task::yield_now().await;
            }
        });
    }
}
