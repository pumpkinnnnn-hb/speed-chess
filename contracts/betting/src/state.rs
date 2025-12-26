use abi::{BetPool, BetRecord, BetSelection, BetStatus, GameOdds, GameResult};
use linera_sdk::linera_base_types::{ApplicationId, ChainId};
use linera_sdk::views::{MapView, RegisterView, RootView, ViewStorageContext};

/// Application state for the Betting contract
#[derive(RootView)]
#[view(context = ViewStorageContext)]
pub struct BettingState {
    /// Bet pools for each game (game_id -> BetPool)
    pub pools: MapView<String, BetPool>,

    /// All bets (bet_id -> BetRecord)
    pub bets: MapView<String, BetRecord>,

    /// User bets lookup (user_chain -> Vec<bet_id>)
    pub user_bets: MapView<ChainId, Vec<String>>,

    /// Game bets lookup (game_id -> Vec<bet_id>)
    pub game_bets: MapView<String, Vec<String>>,

    /// Current odds for each game (game_id -> GameOdds)
    pub odds: MapView<String, GameOdds>,

    /// Counter for generating unique bet IDs
    pub next_bet_id: RegisterView<u64>,

    /// Token application ID
    pub token_app: RegisterView<Option<ApplicationId>>,

    /// Game application ID
    pub game_app: RegisterView<Option<ApplicationId>>,

    /// Minimum bet amount
    pub minimum_bet: RegisterView<u64>,

    /// House edge in basis points (e.g., 200 = 2%)
    pub house_edge: RegisterView<u64>,

    /// Locked games (cannot place new bets)
    pub locked_games: MapView<String, bool>,

    /// Settled games (game_id -> GameResult)
    pub settled_games: MapView<String, GameResult>,
}

impl BettingState {
    /// Generate a new unique bet ID
    pub async fn generate_bet_id(&mut self) -> String {
        let id = *self.next_bet_id.get();
        self.next_bet_id.set(id + 1);
        format!("bet_{:08}", id)
    }

    /// Get or create bet pool for a game
    pub async fn get_or_create_pool(&mut self, game_id: &str) -> Result<BetPool, String> {
        match self.pools.get(game_id).await {
            Ok(Some(pool)) => Ok(pool),
            _ => {
                let pool = BetPool {
                    game_id: game_id.to_string(),
                    total_pool: 0,
                    white_pool: 0,
                    black_pool: 0,
                    draw_pool: 0,
                    is_locked: false,
                };
                self.pools
                    .insert(game_id, pool.clone())
                    .map_err(|e| format!("Failed to create pool: {}", e))?;
                Ok(pool)
            }
        }
    }

    /// Add bet to pool
    pub async fn add_to_pool(
        &mut self,
        game_id: &str,
        bet_on: BetSelection,
        amount: u64,
    ) -> Result<(), String> {
        let mut pool = self.get_or_create_pool(game_id).await?;

        // Check if locked
        if pool.is_locked {
            return Err("Betting is locked for this game".to_string());
        }

        // Add to appropriate pool
        match bet_on {
            BetSelection::White => pool.white_pool += amount,
            BetSelection::Black => pool.black_pool += amount,
            BetSelection::Draw => pool.draw_pool += amount,
        }

        pool.total_pool += amount;

        self.pools
            .insert(game_id, pool)
            .map_err(|e| format!("Failed to update pool: {}", e))?;

        Ok(())
    }

    /// Lock betting for a game
    pub async fn lock_game(&mut self, game_id: &str) -> Result<(), String> {
        let mut pool = self.get_or_create_pool(game_id).await?;
        pool.is_locked = true;
        self.pools
            .insert(game_id, pool)
            .map_err(|e| format!("Failed to lock pool: {}", e))?;

        self.locked_games
            .insert(game_id, true)
            .map_err(|e| format!("Failed to mark game as locked: {}", e))?;

        Ok(())
    }

    /// Store a bet
    pub async fn store_bet(&mut self, bet: BetRecord) -> Result<(), String> {
        let bet_id = bet.id.clone();
        let game_id = bet.game_id.clone();
        let bettor = bet.bettor.clone();

        // Store bet
        self.bets
            .insert(&bet_id, bet)
            .map_err(|e| format!("Failed to store bet: {}", e))?;

        // Add to user's bets
        let bettor_chain = ChainId::from_str(&bettor).map_err(|e| format!("Invalid chain ID: {}", e))?;
        let mut user_bet_list = self
            .user_bets
            .get(&bettor_chain)
            .await
            .map_err(|e| format!("Failed to get user bets: {}", e))?
            .unwrap_or_default();
        user_bet_list.push(bet_id.clone());
        self.user_bets
            .insert(&bettor_chain, user_bet_list)
            .map_err(|e| format!("Failed to update user bets: {}", e))?;

        // Add to game's bets
        let mut game_bet_list = self
            .game_bets
            .get(&game_id)
            .await
            .map_err(|e| format!("Failed to get game bets: {}", e))?
            .unwrap_or_default();
        game_bet_list.push(bet_id.clone());
        self.game_bets
            .insert(&game_id, game_bet_list)
            .map_err(|e| format!("Failed to update game bets: {}", e))?;

        Ok(())
    }

    /// Get bet by ID
    pub async fn get_bet(&self, bet_id: &str) -> Result<Option<BetRecord>, String> {
        self.bets
            .get(bet_id)
            .await
            .map_err(|e| format!("Failed to get bet: {}", e))
    }

    /// Update bet status
    pub async fn update_bet_status(&mut self, bet_id: &str, status: BetStatus) -> Result<(), String> {
        let mut bet = self
            .get_bet(bet_id)
            .await?
            .ok_or("Bet not found")?;

        bet.status = status;

        self.bets
            .insert(bet_id, bet)
            .map_err(|e| format!("Failed to update bet status: {}", e))?;

        Ok(())
    }

    /// Calculate current odds from centipawn evaluation
    pub fn calculate_odds(&self, evaluation: i32) -> GameOdds {
        // Convert centipawn to win probability using logistic function
        // P(win) = 1 / (1 + 10^(-eval/400))

        let eval_f64 = evaluation as f64 / 100.0; // Convert centipawns to pawns

        // Simplified odds calculation
        let white_win_prob = 1.0 / (1.0 + 10_f64.powf(-eval_f64 / 4.0));
        let black_win_prob = 1.0 - white_win_prob;
        let draw_prob = 0.3 * (1.0 - (eval_f64.abs() / 10.0).min(1.0)); // Draw probability decreases with large eval

        // Normalize probabilities
        let total = white_win_prob + black_win_prob + draw_prob;
        let white_norm = white_win_prob / total;
        let black_norm = black_win_prob / total;
        let draw_norm = draw_prob / total;

        // Convert to odds (basis points, 10000 = 1.0x)
        // Odds = 1 / probability * 10000
        let white_odds = if white_norm > 0.01 {
            ((1.0 / white_norm) * 10000.0) as u64
        } else {
            100000 // Cap at 10.0x
        };

        let black_odds = if black_norm > 0.01 {
            ((1.0 / black_norm) * 10000.0) as u64
        } else {
            100000
        };

        let draw_odds = if draw_norm > 0.01 {
            ((1.0 / draw_norm) * 10000.0) as u64
        } else {
            100000
        };

        GameOdds {
            white_odds: white_odds.clamp(10000, 100000), // Between 1.0x and 10.0x
            black_odds: black_odds.clamp(10000, 100000),
            draw_odds: draw_odds.clamp(10000, 100000),
        }
    }

    /// Get all bets for a game
    pub async fn get_game_bets(&self, game_id: &str) -> Result<Vec<BetRecord>, String> {
        let bet_ids = self
            .game_bets
            .get(game_id)
            .await
            .map_err(|e| format!("Failed to get game bets: {}", e))?
            .unwrap_or_default();

        let mut bets = Vec::new();
        for bet_id in bet_ids {
            if let Some(bet) = self.get_bet(&bet_id).await? {
                bets.push(bet);
            }
        }

        Ok(bets)
    }

    /// Calculate winnings for a bet
    pub fn calculate_winnings(&self, bet: &BetRecord, pool: &BetPool, house_edge: u64) -> u64 {
        // Get the winning pool
        let winning_pool = match bet.bet_on {
            BetSelection::White => pool.white_pool,
            BetSelection::Black => pool.black_pool,
            BetSelection::Draw => pool.draw_pool,
        };

        if winning_pool == 0 {
            return 0;
        }

        // Total pool minus house edge
        let total_after_edge = pool.total_pool * (10000 - house_edge) / 10000;

        // User's share of winning pool
        let user_share = (bet.amount as f64) / (winning_pool as f64);

        // Winnings = user's share of total pool after house edge
        (total_after_edge as f64 * user_share) as u64
    }
}

use std::str::FromStr;
