use abi::{ChessGame, ChessMove, GameStatus};
use linera_sdk::linera_base_types::ChainId;
use linera_sdk::views::{MapView, RegisterView, RootView, ViewStorageContext};

/// Application state for the Game contract
#[derive(RootView)]
#[view(context = ViewStorageContext)]
pub struct GameState {
    /// All games indexed by game ID
    pub games: MapView<String, ChessGame>,

    /// Counter for generating unique game IDs
    pub next_game_id: RegisterView<u64>,

    /// Active games lookup (for quick filtering)
    pub active_games: MapView<String, bool>,

    /// Map game ID to player chains (deprecated - now stored in ChessGame)
    // pub game_players: MapView<String, GamePlayers>,

    /// Complete move history for each game
    pub move_history: MapView<String, Vec<ChessMove>>,

    /// Current FEN position for each game (for quick access)
    pub position_fen: MapView<String, String>,

    /// Reference to the Betting Chain (for odds updates)
    pub betting_chain: RegisterView<Option<ChainId>>,

    /// Reference to the Hub Chain (for leaderboard updates)
    pub hub_chain: RegisterView<Option<ChainId>>,
}

impl GameState {
    /// Generate a new unique game ID
    pub async fn generate_game_id(&mut self) -> String {
        let id = *self.next_game_id.get();
        self.next_game_id.set(id + 1);
        format!("game_{:06}", id)
    }

    /// Get a game by ID
    pub async fn get_game(&self, game_id: &str) -> Option<ChessGame> {
        self.games.get(game_id).await.ok().flatten()
    }

    /// Update a game
    pub async fn update_game(&mut self, game: ChessGame) -> Result<(), String> {
        self.games
            .insert(&game.id, game.clone())
            .map_err(|e| format!("Failed to update game: {}", e))?;

        // Update active status
        let is_active = game.status == GameStatus::Active;
        self.active_games
            .insert(&game.id, is_active)
            .map_err(|e| format!("Failed to update active status: {}", e))?;

        Ok(())
    }

    /// Add a move to the history
    pub async fn add_move(&mut self, game_id: &str, chess_move: ChessMove) -> Result<(), String> {
        let mut history = self
            .move_history
            .get(game_id)
            .await
            .map_err(|e| format!("Failed to get move history: {}", e))?
            .unwrap_or_default();

        history.push(chess_move);

        self.move_history
            .insert(game_id, history)
            .map_err(|e| format!("Failed to update move history: {}", e))?;

        Ok(())
    }

    /// Update the current FEN position
    pub async fn update_position(&mut self, game_id: &str, fen: String) -> Result<(), String> {
        self.position_fen
            .insert(game_id, fen)
            .map_err(|e| format!("Failed to update position: {}", e))?;

        Ok(())
    }

    /// Get all active games
    pub async fn get_active_games(&self) -> Result<Vec<ChessGame>, String> {
        let mut active_ids = Vec::new();

        self.active_games
            .for_each_index_value(|game_id, is_active| {
                if *is_active {
                    active_ids.push(game_id.clone());
                }
                Ok(())
            })
            .await
            .map_err(|e| format!("Failed to iterate active games: {}", e))?;


        let mut games = Vec::new();
        for game_id in active_ids {
            if let Some(game) = self.get_game(&game_id).await {
                games.push(game);
            }
        }

        Ok(games)
    }

    /// Get all games (including pending, active, and finished)
    pub async fn get_all_games(&self) -> Result<Vec<ChessGame>, String> {
        let mut games = Vec::new();

        self.games
            .for_each_index_value(|_game_id, game| {
                games.push(game.into_owned());
                Ok(())
            })
            .await
            .map_err(|e| format!("Failed to iterate all games: {}", e))?;

        Ok(games)
    }
}
