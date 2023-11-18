use anyhow::{anyhow, Result};
use axum::{async_trait, extract::ws::{WebSocket, Message}};
use chb_chess::Move;

use super::Participant;

pub struct WebPlayer {
    socket: WebSocket,
}

impl WebPlayer {
    pub fn connect(socket: WebSocket) -> Self {
        Self { socket }
    }

    async fn next(&mut self) -> Result<Message> {
        let mut msg = self
            .socket
            .recv()
            .await
            .ok_or(anyhow!("WebSocket for player closed"))?;
        while msg.is_err() {
            msg = self
                .socket
                .recv()
                .await
                .ok_or(anyhow!("WebSocket for player closed"))?;
        }

        Ok(msg.expect("must be ok"))
    }
}

#[async_trait]
impl Participant for WebPlayer {
    async fn get_move(&mut self) -> Result<Move> {
        match self.next().await? {
            Message::Text(t) => {
                match t.split_once(':') {
                    Some(("move", m)) if m.trim().parse::<Move>().is_ok() => {
                        Ok(m.parse().expect("Validated"))
                    },
                    _ => self.get_move().await,
                }
            },
            _ => self.get_move().await,
        }
    }

    async fn send_move(&mut self, mv: Move) -> Result<()> {
        self.socket.send(Message::Text(format!("move: {mv}"))).await?;
        Ok(())
    }
}
