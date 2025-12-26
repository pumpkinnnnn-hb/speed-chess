use abi::{BetSelection, GameOdds, GameResult};
use linera_sdk::linera_base_types::{ApplicationId, ContractAbi, ServiceAbi};
use linera_sdk::graphql::GraphQLMutationRoot;
use serde::{Deserialize, Serialize};

pub struct BettingAbi;

impl ContractAbi for BettingAbi {
    type Operation = Operation;
    type Response = OperationResult;
}

impl ServiceAbi for BettingAbi {
    type Query = async_graphql::Request;
    type QueryResponse = async_graphql::Response;
}

#[derive(Debug, Clone, Serialize, Deserialize, GraphQLMutationRoot)]
pub enum Operation {
    PlaceBet { game_id: String, bet_on: BetSelection, amount: u64 },
    ClaimWinnings { bet_id: String },
    UpdateOdds { game_id: String, evaluation: i32 },
    SetTokenApp { token_app: ApplicationId },
    SetGameApp { game_app: ApplicationId },
    SetMinimumBet { amount: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    GameStarted { game_id: String },
    PositionUpdated { game_id: String, fen: String, move_count: u32 },
    GameFinished { game_id: String, result: GameResult },
    OddsUpdate { game_id: String, odds: GameOdds },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationResult {
    BetPlaced { bet_id: String, amount: u64, odds: u64 },
    WinningsClaimed { bet_id: String, winnings: u64 },
    OddsUpdated { game_id: String },
    ConfigUpdated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BettingInitializationArgument {
    pub token_app: ApplicationId,
    pub game_app: ApplicationId,
    pub minimum_bet: u64,
    pub house_edge: u64,
}
