use abi::{ChessMove, GameResult};
use linera_sdk::linera_base_types::{ChainId, ContractAbi, ServiceAbi};
use linera_sdk::graphql::GraphQLMutationRoot;
use serde::{Deserialize, Serialize};

pub struct GameAbi;

impl ContractAbi for GameAbi {
    type Operation = Operation;
    type Response = OperationResult;
}

impl ServiceAbi for GameAbi {
    type Query = async_graphql::Request;
    type QueryResponse = async_graphql::Response;
}

#[derive(Debug, Clone, Serialize, Deserialize, GraphQLMutationRoot)]
pub enum Operation {
    CreateGame { opponent_chain: ChainId, time_control: u32 },
    AcceptGame { game_id: String },
    PlaceMove { game_id: String, from: String, to: String, promotion: Option<String> },
    ResignGame { game_id: String },
    OfferDraw { game_id: String },
    AcceptDraw { game_id: String },
    TimeoutGame { game_id: String },
    SetBettingChain { betting_chain: ChainId },
    SetHubChain { hub_chain: ChainId },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    GameInvite { game_id: String },
    OpponentMove { game_id: String, chess_move: ChessMove },
    DrawOffer { game_id: String },
    DrawAccepted { game_id: String },
    GameStarted { game_id: String },
    PositionUpdated { game_id: String, fen: String, move_count: u32 },
    GameFinished { game_id: String, result: GameResult },
    RegisterGame { game_id: String, white_player: ChainId, black_player: ChainId },
    UpdateHubLeaderboard { winner: ChainId },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationResult {
    GameCreated { game_id: String },
    GameAccepted { game_id: String },
    MoveAccepted,
    GameResigned { game_id: String },
    DrawOffered,
    DrawAccepted,
    Timeout { game_id: String },
    ConfigUpdated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEvent {
    GameCreated { game_id: String, white_player: String, black_player: String },
    GameStarted { game_id: String },
    MoveMade { game_id: String, chess_move: ChessMove, new_fen: String },
    GameFinished { game_id: String, result: GameResult },
}
