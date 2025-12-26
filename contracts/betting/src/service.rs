#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use self::state::BettingState;
use async_graphql::{Context, EmptySubscription, Object, Schema};
use betting::{BettingAbi, Operation};
use abi::{BetPool, BetRecord, GameOdds};
use linera_sdk::graphql::GraphQLMutationRoot;
use linera_sdk::linera_base_types::WithServiceAbi;
use linera_sdk::views::{RootView, View};
use linera_sdk::{Service, ServiceRuntime};
use std::sync::Arc;

pub struct BettingService {
    runtime: Arc<ServiceRuntime<Self>>,
}

linera_sdk::service!(BettingService);

impl WithServiceAbi for BettingService {
    type Abi = BettingAbi;
}

impl Service for BettingService {
    type Parameters = ();

    async fn new(runtime: ServiceRuntime<Self>) -> Self {
        BettingService {
            runtime: Arc::new(runtime),
        }
    }

    async fn handle_query(&self, request: Self::Query) -> Self::QueryResponse {
        let state = BettingState::load(self.runtime.root_view_storage_context())
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

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    #[graphql(name = "betPool")]
    async fn bet_pool(&self, ctx: &Context<'_>, #[graphql(name = "gameId")] game_id: String) -> Result<Option<BetPool>, String> {
        let state = ctx.data::<BettingState>().map_err(|e| format!("{:?}", e))?;
        state.pools.get(&game_id).await.map_err(|e| format!("Failed to get pool: {}", e))
    }

    #[graphql(name = "gameOdds")]
    async fn game_odds(&self, ctx: &Context<'_>, #[graphql(name = "gameId")] game_id: String) -> Result<Option<GameOdds>, String> {
        let state = ctx.data::<BettingState>().map_err(|e| format!("{:?}", e))?;
        state.odds.get(&game_id).await.map_err(|e| format!("Failed to get odds: {}", e))
    }

    async fn bet(&self, ctx: &Context<'_>, #[graphql(name = "betId")] bet_id: String) -> Result<Option<BetRecord>, String> {
        let state = ctx.data::<BettingState>().map_err(|e| format!("{:?}", e))?;
        state.get_bet(&bet_id).await
    }

    #[graphql(name = "minimumBet")]
    async fn minimum_bet(&self, ctx: &Context<'_>) -> Result<u64, String> {
        let state = ctx.data::<BettingState>().map_err(|e| format!("{:?}", e))?;
        Ok(*state.minimum_bet.get())
    }

    #[graphql(name = "houseEdge")]
    async fn house_edge(&self, ctx: &Context<'_>) -> Result<u64, String> {
        let state = ctx.data::<BettingState>().map_err(|e| format!("{:?}", e))?;
        Ok(*state.house_edge.get())
    }
}
