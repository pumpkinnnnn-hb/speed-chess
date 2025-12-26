use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};

pub const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, SimpleObject)]
#[graphql(input_name = "ChessMoveInput")]
pub struct ChessMove {
    pub from: String,
    pub to: String,
    pub promotion: Option<String>,
    pub san: String,
    pub piece: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, async_graphql::Enum, Copy)]
pub enum GameResult {
    WhiteWins,
    BlackWins,
    Draw,
    InProgress,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, async_graphql::Enum, Copy)]
pub enum BetSelection {
    White,
    Black,
    Draw,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
#[graphql(input_name = "GameOddsInput")]
pub struct GameOdds {
    pub white_odds: u64,
    pub black_odds: u64,
    pub draw_odds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, async_graphql::Enum, Copy)]
pub enum GameStatus {
    Pending,
    Waiting,
    Active,
    Finished,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct ChessGame {
    pub id: String,
    pub white_player: String,
    pub black_player: String,
    pub fen: String,
    pub current_fen: String,
    pub moves: Vec<ChessMove>,
    pub move_count: u32,
    pub status: GameStatus,
    pub result: Option<GameResult>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, async_graphql::Enum, Copy)]
pub enum BetStatus {
    Pending,
    Won,
    Lost,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct BetRecord {
    pub id: String,
    pub game_id: String,
    pub bettor: String,
    pub amount: u64,
    pub bet_on: BetSelection,
    pub odds: u64,
    pub status: BetStatus,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct BetPool {
    pub game_id: String,
    pub white_pool: u64,
    pub black_pool: u64,
    pub draw_pool: u64,
    pub total_pool: u64,
    pub is_locked: bool,
}
