use chb_chess::Color;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JoinBoard {
    pub id: String,
    pub play_as: Color,
}
