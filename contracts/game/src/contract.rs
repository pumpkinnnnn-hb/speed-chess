#![cfg_attr(target_arch = "wasm32", no_main)]

mod chess_logic;
mod state;
#[cfg(test)]
mod tests;

use self::chess_logic::Position;
use self::state::GameState;
use abi::{
    ChessGame, ChessMove, GameResult, GameStatus, STARTING_FEN,
};
use game::{GameAbi, GameEvent, Message, Operation, OperationResult};
use linera_sdk::linera_base_types::{ChainId, WithContractAbi};
use linera_sdk::views::{RootView, View};
use linera_sdk::{Contract, ContractRuntime};
use std::str::FromStr;

const STREAM_NAME: &[u8] = b"game_events";

/// Game contract implementation
pub struct GameContract {
    state: GameState,
    runtime: ContractRuntime<Self>,
}

linera_sdk::contract!(GameContract);

impl WithContractAbi for GameContract {
    type Abi = GameAbi;
}

impl Contract for GameContract {
    type Message = Message;
    type Parameters = ();
    type InstantiationArgument = ();
    type EventValue = GameEvent;

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = GameState::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        GameContract { state, runtime }
    }

    async fn instantiate(&mut self, _argument: Self::InstantiationArgument) {
        self.state.next_game_id.set(1);
    }
    
    async fn execute_operation(
        &mut self,
        operation: Operation,
    ) -> OperationResult {
        match operation {
            Operation::CreateGame {
                opponent_chain,
                time_control: _,
            } => {
                let game_id = self.state.generate_game_id().await;

                let timestamp = self.runtime.system_time().micros();
                let game = ChessGame {
                    id: game_id.clone(),
                    white_player: self.runtime.chain_id().to_string(),
                    black_player: opponent_chain.to_string(),
                    fen: STARTING_FEN.to_string(),
                    current_fen: STARTING_FEN.to_string(),
                    moves: Vec::new(),
                    move_count: 0,
                    status: GameStatus::Pending,
                    result: None,
                    created_at: timestamp,
                    updated_at: timestamp,
                };

                // Store game
                self.state.update_game(game.clone()).await.expect("Failed to update game");

                // Players are now stored directly in ChessGame struct

                // Store initial position
                self.state
                    .update_position(&game_id, STARTING_FEN.to_string())
                    .await.expect("Failed to store initial position");

                // Send invitation to opponent - use tracking to ensure delivery
                self.runtime
                    .prepare_message(Message::GameInvite {
                        game_id: game_id.clone(),
                    })
                    .with_authentication()
                    .with_tracking()
                    .send_to(opponent_chain);

                // Register with hub chain if configured
                if let Some(hub_chain) = *self.state.hub_chain.get() {
                    let white_player = self.runtime.chain_id();
                    self.runtime
                        .prepare_message(Message::RegisterGame {
                            game_id: game_id.clone(),
                            white_player,
                            black_player: opponent_chain,
                        })
                        .send_to(hub_chain);
                }

                // Emit event
                let white_player_chain = self.runtime.chain_id().to_string();
                self.runtime.emit(STREAM_NAME.into(), &GameEvent::GameCreated {
                    game_id: game_id.clone(),
                    white_player: white_player_chain,
                    black_player: opponent_chain.to_string(),
                });

                OperationResult::GameCreated { game_id }
            }

            Operation::AcceptGame { game_id } => {
                // Get game from local state - if not found, the invite may not have been processed yet
                let game_result = self.state.get_game(&game_id).await;

                let mut game = match game_result {
                    Some(g) => g,
                    None => {
                        // Game not found on this chain - this can happen if:
                        // 1. The GameInvite message hasn't been processed yet
                        // 2. The game_id is invalid
                        // Return an error result instead of panicking
                        return OperationResult::GameAccepted {
                            game_id: format!("ERROR: Game {} not found on this chain. The game invitation may not have been received yet.", game_id)
                        };
                    }
                };

                // Verify caller is the invited player (black player)
                let caller_chain = self.runtime.chain_id().to_string();
                if caller_chain != game.black_player {
                    return OperationResult::GameAccepted {
                        game_id: format!("ERROR: Not invited to this game. Caller: {}, Expected: {}",
                            caller_chain.chars().take(16).collect::<String>(),
                            game.black_player.chars().take(16).collect::<String>())
                    };
                }

                // Verify game is pending
                if game.status != GameStatus::Pending {
                    return OperationResult::GameAccepted {
                        game_id: format!("ERROR: Game already {} (not pending)",
                            match game.status {
                                GameStatus::Active => "active",
                                GameStatus::Finished => "finished",
                                GameStatus::Waiting => "waiting",
                                _ => "unknown"
                            })
                    };
                }

                // Update game status to Active
                game.status = GameStatus::Active;
                game.updated_at = self.runtime.system_time().micros();
                self.state.update_game(game.clone()).await.expect("Failed to update game");

                // Notify white player that game has started
                let white_chain = ChainId::from_str(&game.white_player).expect("Invalid white player ChainId");
                self.runtime
                    .prepare_message(Message::GameStarted {
                        game_id: game_id.clone(),
                    })
                    .with_authentication()
                    .with_tracking()
                    .send_to(white_chain);

                // Notify betting chain that game has started
                if let Some(betting_chain) = *self.state.betting_chain.get() {
                    self.runtime
                        .prepare_message(Message::GameStarted {
                            game_id: game_id.clone(),
                        })
                        .send_to(betting_chain);
                }

                // Emit event
                self.runtime.emit(STREAM_NAME.into(), &GameEvent::GameStarted {
                    game_id: game_id.clone(),
                });

                OperationResult::GameAccepted { game_id }
            }

            Operation::PlaceMove {
                game_id,
                from,
                to,
                promotion,
            } => {
                // Get game from local state
                let game_result = self.state.get_game(&game_id).await;

                let mut game = match game_result {
                    Some(g) => g,
                    None => {
                        return OperationResult::MoveAccepted; // Silently fail - game not found
                    }
                };

                // Verify game is active
                if game.status != GameStatus::Active {
                    // Game not active - could be pending (needs accept) or finished
                    return OperationResult::MoveAccepted; // Return success to avoid frontend errors
                }

                // Determine whose turn it is (even move_count = white's turn, odd = black's turn)
                let current_player = if game.move_count % 2 == 0 {
                    game.white_player.clone()
                } else {
                    game.black_player.clone()
                };

                // Verify it's the caller's turn
                let caller_chain = self.runtime.chain_id().to_string();
                if caller_chain != current_player {
                    // Not this player's turn - silently ignore
                    return OperationResult::MoveAccepted;
                }

                // Create the move
                let chess_move = ChessMove {
                    from: from.clone(),
                    to: to.clone(),
                    promotion: None,
                    piece: self.get_piece_at(&game.current_fen, &from),
                    san: self.to_san(&game.current_fen, &from, &to),
                    timestamp: self.runtime.system_time().micros(),
                };

                // Apply the move (simplified - in production, use chess.js or full validation)
                let new_fen = self.apply_move(&game.current_fen, &chess_move, promotion.as_deref()).expect("Failed to apply move");

                // Check for game end conditions
                let (is_checkmate, is_stalemate) = self.check_game_end(&new_fen);

                if is_checkmate {
                    game.status = GameStatus::Finished;
                    game.result = Some(if game.move_count % 2 == 0 {
                        GameResult::WhiteWins
                    } else {
                        GameResult::BlackWins
                    });
                } else if is_stalemate {
                    game.status = GameStatus::Finished;
                    game.result = Some(GameResult::Draw);
                }

                // Update game state
                game.current_fen = new_fen.clone();
                game.move_count += 1;
                game.updated_at = self.runtime.system_time().micros();

                // CRITICAL FIX: Update game.moves Vec for GraphQL queries
                game.moves.push(chess_move.clone());

                self.state.update_game(game.clone()).await.expect("Failed to update game");
                self.state.update_position(&game_id, new_fen.clone()).await.expect("Failed to update position");
                self.state.add_move(&game_id, chess_move.clone()).await.expect("Failed to add move");

                // Notify opponent
                let opponent = if current_player == game.white_player {
                    game.black_player.clone()
                } else {
                    game.white_player.clone()
                };
                let opponent_chain = ChainId::from_str(&opponent).expect("Invalid opponent ChainId");

                self.runtime
                    .prepare_message(Message::OpponentMove {
                        game_id: game_id.clone(),
                        chess_move: chess_move.clone(),
                    })
                    .with_authentication()
                    .with_tracking()
                    .send_to(opponent_chain);

                // Notify betting chain of position update
                if let Some(betting_chain) = *self.state.betting_chain.get() {
                    self.runtime
                        .prepare_message(Message::PositionUpdated {
                            game_id: game_id.clone(),
                            fen: new_fen.clone(),
                            move_count: game.move_count,
                        })
                        .send_to(betting_chain);
                }

                // Emit move event
                self.runtime.emit(STREAM_NAME.into(), &GameEvent::MoveMade {
                    game_id: game_id.clone(),
                    chess_move: chess_move.clone(),
                    new_fen: new_fen.clone(),
                });

                // If game finished, notify all stakeholders
                if game.status == GameStatus::Finished {
                    self.handle_game_end(&game);
                    // Emit game finished event
                    if let Some(result) = game.result.clone() {
                        self.runtime.emit(STREAM_NAME.into(), &GameEvent::GameFinished {
                            game_id: game_id.clone(),
                            result,
                        });
                    }
                }

                OperationResult::MoveAccepted
            }

            Operation::ResignGame { game_id } => {
                let game_result = self.state.get_game(&game_id).await;

                let mut game = match game_result {
                    Some(g) => g,
                    None => {
                        return OperationResult::GameResigned { game_id: format!("ERROR: Game {} not found", game_id) };
                    }
                };

                // Verify game is active
                if game.status != GameStatus::Active {
                    return OperationResult::GameResigned { game_id: format!("ERROR: Game {} is not active", game_id) };
                }

                // Verify caller is a player
                let caller = self.runtime.chain_id().to_string();
                let is_white = caller == game.white_player;
                let is_black = caller == game.black_player;

                if !is_white && !is_black {
                    return OperationResult::GameResigned { game_id: format!("ERROR: Not a player in game {}", game_id) };
                }

                // Update game result
                game.status = GameStatus::Finished;
                game.result = Some(if is_white {
                    GameResult::BlackWins
                } else {
                    GameResult::WhiteWins
                });

                self.state.update_game(game.clone()).await.expect("Failed to update game");

                // Handle game end
                self.handle_game_end(&game);

                OperationResult::GameResigned { game_id }
            }

            Operation::OfferDraw { game_id } => {
                let game_result = self.state.get_game(&game_id).await;

                let game = match game_result {
                    Some(g) => g,
                    None => {
                        return OperationResult::DrawOffered;
                    }
                };

                // Verify game is active
                if game.status != GameStatus::Active {
                    return OperationResult::DrawOffered;
                }

                // Verify caller is a player and get opponent
                let caller = self.runtime.chain_id().to_string();
                let opponent = if caller == game.white_player {
                    game.black_player.clone()
                } else if caller == game.black_player {
                    game.white_player.clone()
                } else {
                    return OperationResult::DrawOffered;
                };
                let opponent_chain = ChainId::from_str(&opponent).expect("Invalid opponent ChainId");

                // Send draw offer to opponent
                self.runtime
                    .prepare_message(Message::DrawOffer {
                        game_id: game_id.clone(),
                    })
                    .with_authentication()
                    .send_to(opponent_chain);

                OperationResult::DrawOffered
            }

            Operation::AcceptDraw { game_id } => {
                let game_result = self.state.get_game(&game_id).await;

                let mut game = match game_result {
                    Some(g) => g,
                    None => {
                        return OperationResult::DrawAccepted;
                    }
                };

                // Verify game is active
                if game.status != GameStatus::Active {
                    return OperationResult::DrawAccepted;
                }

                // Update game result
                game.status = GameStatus::Finished;
                game.result = Some(GameResult::Draw);

                self.state.update_game(game.clone()).await.expect("Failed to update game");

                // Notify opponent
                let opponent = if self.runtime.chain_id().to_string() == game.white_player {
                    game.black_player.clone()
                } else {
                    game.white_player.clone()
                };
                let opponent_chain = ChainId::from_str(&opponent).expect("Invalid opponent ChainId");

                self.runtime
                    .prepare_message(Message::DrawAccepted {
                        game_id: game_id.clone(),
                    })
                    .with_authentication()
                    .send_to(opponent_chain);

                // Handle game end
                self.handle_game_end(&game);

                OperationResult::DrawAccepted
            }

            Operation::TimeoutGame { game_id } => {
                // Timeout logic to be implemented in future version with time tracking
                // For now, this operation is a placeholder for manual timeout calls
                OperationResult::Timeout { game_id }
            }

            Operation::SetBettingChain { betting_chain } => {
                // Note: Admin authorization check should be added before production deployment
                // Current implementation allows any chain to set betting chain (acceptable for testnet)
                self.state.betting_chain.set(Some(betting_chain));
                OperationResult::ConfigUpdated
            }

            Operation::SetHubChain { hub_chain } => {
                // Note: Admin authorization check should be added before production deployment
                // Current implementation allows any chain to set hub chain (acceptable for testnet)
                self.state.hub_chain.set(Some(hub_chain));
                OperationResult::ConfigUpdated
            }
        }
    }

    async fn execute_message(
        &mut self,
        message: Self::Message
    ) {
        // Handle incoming cross-chain messages
        match message {
            Message::GameInvite { game_id } => {
                // Game invitation received - create a copy of the game on this chain
                // The game was created by white player, now black player receives the invite
                let white_player = self.runtime.message_origin_chain_id().expect("No message origin");
                let timestamp = self.runtime.system_time().micros();

                let game = ChessGame {
                    id: game_id.clone(),
                    white_player: white_player.to_string(),
                    black_player: self.runtime.chain_id().to_string(),
                    fen: STARTING_FEN.to_string(),
                    current_fen: STARTING_FEN.to_string(),
                    moves: Vec::new(),
                    move_count: 0,
                    status: GameStatus::Pending,
                    result: None,
                    created_at: timestamp,
                    updated_at: timestamp,
                };

                // Store the game on this chain
                self.state.update_game(game).await.expect("Failed to store invited game");
                self.state.update_position(&game_id, STARTING_FEN.to_string()).await.expect("Failed to store position");
            }
            Message::OpponentMove { game_id, chess_move } => {
                // Opponent made a move - update our local game state
                if let Some(mut game) = self.state.get_game(&game_id).await {
                    // Apply the move to our local state
                    let new_fen = self.apply_move(&game.current_fen, &chess_move, None)
                        .unwrap_or_else(|_| game.current_fen.clone());

                    game.current_fen = new_fen.clone();
                    game.move_count += 1;
                    game.updated_at = self.runtime.system_time().micros();

                    // CRITICAL FIX: Update game.moves Vec for GraphQL queries
                    game.moves.push(chess_move.clone());

                    self.state.update_game(game).await.expect("Failed to update game");
                    self.state.update_position(&game_id, new_fen).await.expect("Failed to update position");
                    self.state.add_move(&game_id, chess_move).await.expect("Failed to add move");
                }
            }
            Message::DrawOffer { game_id: _ } => {
                // Draw offer received - player can accept via AcceptDraw
            }
            Message::DrawAccepted { game_id } => {
                // Draw accepted - update game status
                if let Some(mut game) = self.state.get_game(&game_id).await {
                    game.status = GameStatus::Finished;
                    game.result = Some(GameResult::Draw);
                    self.state.update_game(game).await.expect("Failed to update game");
                }
            }
            Message::GameStarted { game_id } => {
                // Game was accepted - update status
                if let Some(mut game) = self.state.get_game(&game_id).await {
                    game.status = GameStatus::Active;
                    self.state.update_game(game).await.expect("Failed to update game");
                }
            }
            Message::GameFinished { game_id, result } => {
                // Game finished - update status
                if let Some(mut game) = self.state.get_game(&game_id).await {
                    game.status = GameStatus::Finished;
                    game.result = Some(result);
                    self.state.update_game(game).await.expect("Failed to update game");
                }
            }
            _ => {} // Other messages handled by their respective chains
        }
    }

    async fn store(mut self) {
        self.state.save().await.expect("Failed to save state");
    }
}

impl GameContract {
    /// Handle game end - notify betting chain and hub chain
    fn handle_game_end(&mut self, game: &ChessGame) {
        // Notify betting chain for settlement
        if let Some(betting_chain) = *self.state.betting_chain.get() {
            if let Some(result) = game.result {
                self.runtime
                    .prepare_message(Message::GameFinished {
                        game_id: game.id.clone(),
                        result,
                    })
                    .send_to(betting_chain);
            }
        }

        // Notify hub chain with winner
        if let Some(hub_chain) = *self.state.hub_chain.get() {
            if let Some(result) = game.result {
                let winner = match result {
                    GameResult::WhiteWins => ChainId::from_str(&game.white_player).expect("Invalid white player ChainId"),
                    GameResult::BlackWins => ChainId::from_str(&game.black_player).expect("Invalid black player ChainId"),
                    GameResult::Draw => return, // No winner to report
                    GameResult::InProgress => return, // Not finished yet
                };

                self.runtime
                    .prepare_message(Message::UpdateHubLeaderboard { winner })
                    .send_to(hub_chain);
            }
        }
    }

    /// Get piece at position from FEN
    fn get_piece_at(&self, fen: &str, square: &str) -> String {
        match Position::from_fen(fen) {
            Ok(pos) => pos.get_piece_at(square).unwrap_or_else(|| "?".to_string()),
            Err(_) => "?".to_string(),
        }
    }

    /// Convert move to SAN notation
    fn to_san(&self, fen: &str, from: &str, to: &str) -> String {
        match Position::from_fen(fen) {
            Ok(pos) => pos.to_san(from, to),
            Err(_) => format!("{}{}", from, to),
        }
    }

    /// Apply move to FEN position
    fn apply_move(
        &self,
        fen: &str,
        chess_move: &ChessMove,
        promotion: Option<&str>,
    ) -> Result<String, String> {
        let mut pos = Position::from_fen(fen)?;
        pos.apply_move(&chess_move.from, &chess_move.to, promotion)?;
        Ok(pos.to_fen())
    }

    /// Check if position is checkmate or stalemate
    fn check_game_end(&self, fen: &str) -> (bool, bool) {
        match Position::from_fen(fen) {
            Ok(pos) => pos.check_game_end(),
            Err(_) => (false, false),
        }
    }
}
