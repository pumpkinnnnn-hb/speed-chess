#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;
#[cfg(test)]
mod tests;

use self::state::BettingState;
use betting::{BettingAbi, BettingInitializationArgument, Message, Operation, OperationResult};
use abi::{BetRecord, BetSelection, BetStatus};
use linera_sdk::linera_base_types::WithContractAbi;
use linera_sdk::views::{RootView, View, ViewStorageContext};
use linera_sdk::{Contract, ContractRuntime};

pub struct BettingContract {
    state: BettingState,
    runtime: ContractRuntime<Self>,
}

linera_sdk::contract!(BettingContract);

impl WithContractAbi for BettingContract {
    type Abi = BettingAbi;
}

impl Contract for BettingContract {
    type Message = Message;
    type Parameters = ();
    type InstantiationArgument = BettingInitializationArgument;
    type EventValue = ();

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = BettingState::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        BettingContract { state, runtime }
    }

    async fn instantiate(&mut self, argument: Self::InstantiationArgument) {
        assert!(argument.minimum_bet > 0, "Minimum bet must be greater than zero");
        assert!(argument.house_edge <= 1000, "House edge cannot exceed 10%");
        self.state.token_app.set(Some(argument.token_app));
        self.state.game_app.set(Some(argument.game_app));
        self.state.minimum_bet.set(argument.minimum_bet);
        self.state.house_edge.set(argument.house_edge);
        self.state.next_bet_id.set(1);
    }

    async fn execute_operation(&mut self, operation: Self::Operation) -> Self::Response {
        match operation {
            Operation::PlaceBet {
                game_id,
                bet_on,
                amount,
            } => {
                let min_bet = *self.state.minimum_bet.get();
                assert!(amount >= min_bet, "Bet amount is below minimum");

                if let Ok(Some(true)) = self.state.locked_games.get(&game_id).await {
                    panic!("Betting is locked for this game");
                }

                let odds = self.state.odds.get(&game_id).await
                    .expect("Failed to get odds")
                    .expect("No odds available for this game");

                let odds_value = match bet_on {
                    BetSelection::White => odds.white_odds,
                    BetSelection::Black => odds.black_odds,
                    BetSelection::Draw => odds.draw_odds,
                };

                let bet_id = self.state.generate_bet_id().await;
                let bet = BetRecord {
                    id: bet_id.clone(),
                    game_id: game_id.clone(),
                    bettor: self.runtime.chain_id().to_string(),
                    amount,
                    bet_on,
                    odds: odds_value,
                    status: BetStatus::Pending,
                    timestamp: self.runtime.system_time().micros(),
                };

                self.state.store_bet(bet).await.expect("Failed to store bet");
                self.state.add_to_pool(&game_id, bet_on, amount).await.expect("Failed to add to pool");

                OperationResult::BetPlaced { bet_id, amount, odds: odds_value }
            }

            Operation::ClaimWinnings { bet_id } => {
                let bet = self.state.get_bet(&bet_id).await
                    .expect("Failed to get bet")
                    .expect("Bet not found");

                assert!(bet.bettor == self.runtime.chain_id().to_string(), "Not your bet");
                assert!(bet.status == BetStatus::Won, "Bet is not in Won status");

                let _result = self.state.settled_games.get(&bet.game_id).await
                    .expect("Failed to get game result")
                    .expect("Game not settled");

                let pool = self.state.get_or_create_pool(&bet.game_id).await.expect("Failed to get pool");
                let house_edge = *self.state.house_edge.get();
                let winnings = self.state.calculate_winnings(&bet, &pool, house_edge);

                OperationResult::WinningsClaimed { bet_id, winnings }
            }

            Operation::UpdateOdds { game_id, evaluation } => {
                let odds = self.state.calculate_odds(evaluation);
                // Odds updated (last_updated removed from GameOdds)
                self.state.odds.insert(&game_id, odds).expect("Failed to update odds");
                OperationResult::OddsUpdated { game_id }
            }

            Operation::SetTokenApp { token_app } => {
                self.state.token_app.set(Some(token_app));
                OperationResult::ConfigUpdated
            }

            Operation::SetGameApp { game_app } => {
                self.state.game_app.set(Some(game_app));
                OperationResult::ConfigUpdated
            }

            Operation::SetMinimumBet { amount } => {
                assert!(amount > 0, "Minimum bet must be greater than zero");
                self.state.minimum_bet.set(amount);
                OperationResult::ConfigUpdated
            }
        }
    }

    async fn execute_message(&mut self, message: Self::Message) {
        match message {
            Message::GameStarted { game_id } => {
                self.state.lock_game(&game_id).await.expect("Failed to lock game");
            }
            Message::PositionUpdated { game_id: _, fen: _, move_count: _ } => {}
            Message::GameFinished { game_id, result } => {
                self.settle_game(&game_id, result).await.expect("Failed to settle game");
            }
            Message::OddsUpdate { game_id, odds } => {
                self.state.odds.insert(&game_id, odds).expect("Failed to update odds");
            }
        }
    }

    async fn store(mut self) {
        self.state.save().await.expect("Failed to save state");
    }
}

impl BettingContract {
    async fn settle_game(&mut self, game_id: &str, result: abi::GameResult) -> Result<(), String> {
        self.state.settled_games.insert(game_id, result).map_err(|e| format!("Failed to store result: {}", e))?;
        let bets = self.state.get_game_bets(game_id).await?;
        let winning_selection = match result {
            abi::GameResult::WhiteWins => BetSelection::White,
            abi::GameResult::BlackWins => BetSelection::Black,
            abi::GameResult::Draw => BetSelection::Draw,
            abi::GameResult::InProgress => return Ok(()), // Game not finished
        };
        for bet in bets {
            let new_status = if bet.bet_on == winning_selection { BetStatus::Won } else { BetStatus::Lost };
            self.state.update_bet_status(&bet.id, new_status).await?;
        }
        Ok(())
    }
}
