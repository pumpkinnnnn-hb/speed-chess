#![cfg_attr(target_arch = "wasm32", no_main)]

use linera_sdk::{Contract, ContractRuntime};
use linera_sdk::linera_base_types::WithContractAbi;
use linera_sdk::views::{RootView, View, ViewStorageContext};
use async_graphql::{Request, Response};

pub struct GameContract {
    runtime: ContractRuntime<Self>,
}

linera_sdk::contract!(GameContract);

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize)]
pub struct GameAbi;

impl WithContractAbi for GameContract {
    type Abi = GameAbi;
}

#[async_graphql::SimpleObject]
struct Dummy {
    value: String,
}

impl Contract for GameContract {
    type Message = ();
    type Parameters = ();
    type InstantiationArgument = ();
   type EventValue = ();

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        GameContract { runtime }
    }

    async fn instantiate(&mut self, _argument: ()) {
        // Minimal initialization
    }

    async fn execute_operation(&mut self, _operation: ()) -> () {
        // Stub
    }

    async fn execute_message(&mut self, _message: ()) {
        // Stub
    }

    async fn store(mut self) {
        // Stub
    }
}

pub struct GameService {
    runtime: ContractRuntime<GameContract>,
}

#[async_graphql::Object]
impl GameService {
    async fn dummy(&self) -> Dummy {
        Dummy { value: "stub".to_string() }
    }
}
