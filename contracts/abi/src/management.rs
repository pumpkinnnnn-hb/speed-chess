use crate::leaderboard::SimpleLeaderboardEntry;
use crate::player_dealer::{Dealer, Player};
use async_graphql::scalar;
use async_graphql_derive::SimpleObject;
use linera_sdk::linera_base_types::{AccountOwner, ChainId, Timestamp};
use serde::{Deserialize, Serialize};

pub type TournamentId = u64;

pub type RoomId = u64;

scalar!(ActivityStatus);
#[derive(Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize)]
#[repr(u8)]
pub enum ActivityStatus {
    #[default]
    Active = 0,
    Inactive = 1,
    MaintenanceMode = 2,
    TournamentExclusive {
        tournament_id: TournamentId,
    } = 3,
}

scalar!(ManagedBy);
#[derive(Debug, Clone, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize)]
#[repr(u8)]
pub enum ManagedBy {
    Public { chain_id: ChainId } = 0,
    User { chain_id: ChainId } = 1,
}

// * ----------------------------------------------------------------------------------------------------
// * Public Chain
// * ----------------------------------------------------------------------------------------------------

scalar!(PublicChainType);
#[derive(Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize)]
#[repr(u8)]
pub enum PublicChainType {
    #[default]
    Regular = 0,
    Tournament {
        tournament_id: TournamentId,
    } = 1,
}

#[derive(Debug, Clone, Default, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct PublicChainInfo {
    pub chain_id: Option<ChainId>,
    pub chain_status: ActivityStatus,
    pub chain_type: PublicChainType,
    pub created_at: Option<Timestamp>,
    pub last_update: Option<Timestamp>,
}

impl PublicChainInfo {
    pub fn new(chain_id: ChainId, current_time: Timestamp) -> Self {
        PublicChainInfo {
            chain_id: Some(chain_id),
            chain_status: ActivityStatus::Active,
            chain_type: PublicChainType::Regular,
            created_at: Some(current_time),
            last_update: Some(current_time),
        }
    }
}

// * ----------------------------------------------------------------------------------------------------
// * Room
// * ----------------------------------------------------------------------------------------------------

scalar!(RoomType);
#[derive(Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize)]
#[repr(u8)]
pub enum RoomType {
    #[default]
    Public = 0,
    Private {
        password_hash: String,
    } = 1,
    Tournament {
        tournament_id: TournamentId,
    } = 2,
}

#[derive(Debug, Clone, Default, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct RoomInfo {
    pub chain_id: Option<ChainId>,
    pub chain_owner: Option<AccountOwner>,
    pub room_id: RoomId,
    pub name: String,
    pub room_type: RoomType,
    pub room_status: ActivityStatus,
    pub room_history: Option<RoomHistory>,
    pub managed_by: Option<ManagedBy>,
    pub game_count: u64,
    pub current_players: u8,
    pub current_spectators: u64,
    pub created_at: Option<Timestamp>,
    pub last_update: Option<Timestamp>,
    pub leaderboard_update: Option<Vec<SimpleLeaderboardEntry>>,
    pub owner_list: Vec<AccountOwner>,
}

impl RoomInfo {
    pub fn new(
        chain_id: ChainId,
        chain_owner: Option<AccountOwner>,
        name: String,
        room_type: RoomType,
        managed_by: ManagedBy,
        current_time: Timestamp,
    ) -> Self {
        RoomInfo {
            chain_id: Some(chain_id),
            chain_owner,
            room_id: current_time.micros(),
            name,
            room_type,
            managed_by: Some(managed_by),
            room_status: ActivityStatus::Active,
            room_history: None,
            game_count: 0,
            current_players: 0,
            current_spectators: 0,
            created_at: Some(current_time),
            last_update: Some(current_time),
            leaderboard_update: None,
            owner_list: Vec::new(),
        }
    }

    pub fn hand_count_update(&mut self, current_players: u8, game_count: u64, current_time: Timestamp, room_history: RoomHistory) {
        self.game_count = game_count;
        self.room_history = Some(room_history);
        self.current_players = current_players;
        self.last_update = Some(current_time);
    }

    fn data_for_update(&self, leaderboard_update: Option<Vec<SimpleLeaderboardEntry>>) -> Self {
        RoomInfo {
            chain_id: self.chain_id,
            chain_owner: self.chain_owner,
            room_id: self.room_id,
            name: self.name.clone(),
            room_type: self.room_type.clone(),
            managed_by: self.managed_by.clone(),
            room_status: self.room_status.clone(),
            room_history: None,
            game_count: self.game_count,
            current_players: self.current_players,
            current_spectators: self.current_spectators,
            created_at: self.created_at,
            last_update: self.last_update,
            leaderboard_update,
            owner_list: self.owner_list.clone(),
        }
    }

    pub fn data_for_event(&self, leaderboard: Vec<SimpleLeaderboardEntry>) -> Self {
        self.data_for_update(Some(leaderboard))
    }

    pub fn data_for_process_update(&self) -> Self {
        self.data_for_update(None)
    }

    /// Check if an AccountOwner exists in the owner_list
    pub fn has_owner(&self, owner: &AccountOwner) -> bool {
        self.owner_list.contains(owner)
    }

    /// Add an AccountOwner to the owner_list if not already present
    pub fn add_owner(&mut self, owner: AccountOwner) {
        if !self.has_owner(&owner) {
            self.owner_list.push(owner);
        }
    }

    /// Remove an AccountOwner from the owner_list
    /// Returns true if the owner was found and removed, false otherwise
    pub fn remove_owner(&mut self, owner: &AccountOwner) -> bool {
        if let Some(pos) = self.owner_list.iter().position(|o| o == owner) {
            self.owner_list.remove(pos);
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct RoomHistory {
    pub dealer: Dealer,
    pub players: Vec<Player>,
}

impl RoomHistory {
    pub fn new(dealer: Dealer, players: Vec<Player>) -> Self {
        RoomHistory { dealer, players }
    }
}
