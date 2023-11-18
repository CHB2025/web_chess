use anyhow::Result;
use axum::async_trait;
use chb_chess::Move;

pub mod web_player;

#[async_trait]
pub trait Participant {
    async fn get_move(&mut self) -> Result<Move>; // Cannot send error responses, but oh well
    async fn send_move(&mut self, mv: Move) -> Result<()>;
}
