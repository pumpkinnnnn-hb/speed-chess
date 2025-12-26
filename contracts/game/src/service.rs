#![cfg_attr(target_arch = "wasm32", no_main)]

mod chess_logic;
mod state;

use self::state::GameState;
use async_graphql::{Context, EmptySubscription, Object, Schema};
use abi::{ChessGame, ChessMove};
use game::{GameAbi, Operation};
use linera_sdk::graphql::GraphQLMutationRoot;
use linera_sdk::linera_base_types::WithServiceAbi;
use linera_sdk::views::{RootView, View};
use linera_sdk::{Service, ServiceRuntime};
use std::sync::Arc;

/// Game service for GraphQL queries
pub struct GameService {
    runtime: Arc<ServiceRuntime<Self>>,
}

linera_sdk::service!(GameService);

impl WithServiceAbi for GameService {
    type Abi = GameAbi;
}

impl Service for GameService {
    type Parameters = ();

    async fn new(runtime: ServiceRuntime<Self>) -> Self {
        GameService {
            runtime: Arc::new(runtime),
        }
    }

    async fn handle_query(&self, request: Self::Query) -> Self::QueryResponse {
        let state = GameState::load(self.runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");

        let schema = Schema::build(
            QueryRoot,
            Operation::mutation_root(self.runtime.clone()),
            EmptySubscription
        )
            .data(state)
            .finish();
        schema.execute(request).await
    }
}

/// GraphQL query root
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get a game by ID
    async fn game(&self, ctx: &Context<'_>, #[graphql(name = "gameId")] id: String) -> Result<Option<ChessGame>, String> {
        let state = ctx.data::<GameState>().map_err(|e| format!("{:?}", e))?;
        Ok(state.get_game(&id).await)
    }

    /// Get all active games
    #[graphql(name = "activeGames")]
    async fn active_games(&self, ctx: &Context<'_>) -> Result<Vec<ChessGame>, String> {
        let state = ctx.data::<GameState>().map_err(|e| format!("{:?}", e))?;
        state.get_active_games().await
    }

    /// Get all games (including pending)
    #[graphql(name = "allGames")]
    async fn all_games(&self, ctx: &Context<'_>) -> Result<Vec<ChessGame>, String> {
        let state = ctx.data::<GameState>().map_err(|e| format!("{:?}", e))?;
        state.get_all_games().await
    }

    /// Get move history for a game
    #[graphql(name = "moveHistory")]
    async fn move_history(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "gameId")] game_id: String,
    ) -> Result<Vec<ChessMove>, String> {
        let state = ctx.data::<GameState>().map_err(|e| format!("{:?}", e))?;
        state
            .move_history
            .get(&game_id)
            .await
            .map_err(|e| format!("Failed to get move history: {}", e))?
            .ok_or_else(|| "No move history found".to_string())
    }

    /// Get current FEN position for a game
    async fn position(&self, ctx: &Context<'_>, #[graphql(name = "gameId")] game_id: String) -> Result<String, String> {
        let state = ctx.data::<GameState>().map_err(|e| format!("{:?}", e))?;
        state
            .position_fen
            .get(&game_id)
            .await
            .map_err(|e| format!("Failed to get position: {}", e))?
            .ok_or_else(|| "No position found".to_string())
    }

    /// Get the last move for a game
    #[graphql(name = "lastMove")]
    async fn last_move(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "gameId")] game_id: String,
    ) -> Result<Option<ChessMove>, String> {
        let state = ctx.data::<GameState>().map_err(|e| format!("{:?}", e))?;
        let history = state
            .move_history
            .get(&game_id)
            .await
            .map_err(|e| format!("Failed to get move history: {}", e))?
            .unwrap_or_default();

        Ok(history.last().cloned())
    }
}
