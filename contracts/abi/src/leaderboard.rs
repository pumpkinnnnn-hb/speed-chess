use crate::bet_chip_profile::PlayerLifetimeStatistics;
use async_graphql::scalar;
use async_graphql_derive::SimpleObject;
use linera_sdk::linera_base_types::{Amount, ChainId};
use serde::{Deserialize, Serialize};

scalar!(SimpleRankingMetric);
#[derive(Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize)]
pub enum SimpleRankingMetric {
    /// Net profit using composite ranking: profits rank above losses.
    /// Ranking value: profit -> (1 << 127) + amount, loss -> (1 << 127) - 1 - amount
    #[default]
    NetProfit,
    /// Gross winnings in attos (ignores losses)
    TotalWinnings,
    /// Win rate in basis points (10000 = 100%)
    WinRate,
    /// Total games played (activity-based)
    GamesPlayed,
    /// Total hands played (activity-based, more granular)
    HandsPlayed,
    /// Current win streak
    CurrentStreak,
    /// Number of blackjacks hit
    BlackjackCount,
}

/// Offset used for composite ranking of profit/loss values.
/// Profits: (1 << 127) + profit (upper half of u128 range)
/// Losses: (1 << 127) - 1 - loss (lower half, smaller loss = higher value)
const RANKING_MIDPOINT: u128 = 1u128 << 127;

impl SimpleRankingMetric {
    /// Calculate the metric value for a player's statistics.
    /// All calculations use unsigned integers only (no floats or signed integers).
    ///
    /// For NetProfit, uses composite ranking:
    /// - Profits: MIDPOINT + profit (ranks higher, larger profit = higher rank)
    /// - Losses: MIDPOINT - 1 - loss (ranks lower, smaller loss = higher rank)
    pub fn calculate_value(&self, stats: &PlayerLifetimeStatistics) -> u128 {
        match self {
            SimpleRankingMetric::NetProfit => {
                let won_attos = stats.total_won.to_attos();
                let lost_attos = stats.total_lost.to_attos();

                if won_attos >= lost_attos {
                    // Profit: place in upper half of u128 range
                    let profit = won_attos.saturating_sub(lost_attos);
                    RANKING_MIDPOINT.saturating_add(profit)
                } else {
                    // Loss: place in lower half, smaller loss = higher value
                    let loss = lost_attos.saturating_sub(won_attos);
                    (RANKING_MIDPOINT - 1).saturating_sub(loss)
                }
            }
            SimpleRankingMetric::TotalWinnings => stats.total_won.to_attos(),
            SimpleRankingMetric::WinRate => {
                if stats.total_hands == 0 {
                    0
                } else {
                    // Win rate in basis points (10000 = 100%)
                    (stats.hands_won as u128).saturating_mul(10000) / (stats.total_hands as u128)
                }
            }
            SimpleRankingMetric::GamesPlayed => stats.total_games as u128,
            SimpleRankingMetric::HandsPlayed => stats.total_hands as u128,
            SimpleRankingMetric::CurrentStreak => stats.current_win_streak as u128,
            SimpleRankingMetric::BlackjackCount => stats.blackjacks_hit as u128,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize, SimpleObject)]
pub struct SimpleLeaderboardEntry {
    pub player_id: Option<ChainId>,
    pub player_name: String,
    pub rank: u32,
    pub metric_type: SimpleRankingMetric,

    // Net profit as (amount, is_profit) tuple representation
    pub net_profit_amount: Amount,
    pub is_profit: bool, // true = profit, false = loss

    // Other calculated metric values
    pub total_winnings: Amount,
    pub win_rate: u64, // Basis points (0-10000)
    pub games_played: u64,
    pub hands_played: u64,
    pub current_streak: u64,
    pub blackjack_count: u64,
}

/// Extract net profit as (absolute_amount, is_profit) tuple from player statistics.
///
/// # Returns
/// - `(amount, true)` if player has profit or break-even
/// - `(amount, false)` if player has net loss
fn calculate_net_profit_tuple(stats: &PlayerLifetimeStatistics) -> (u128, bool) {
    let won_attos = stats.total_won.to_attos();
    let lost_attos = stats.total_lost.to_attos();

    if won_attos >= lost_attos {
        (won_attos - lost_attos, true) // Profit
    } else {
        (lost_attos - won_attos, false) // Loss
    }
}

/// Calculate a ranked leaderboard from player statistics.
///
/// # Arguments
/// * `player_stats` - Vec of (ChainId, PlayerLifetimeStatistics) tuples
/// * `metric` - The ranking metric to use
/// * `limit` - Maximum number of entries to return (0 = unlimited)
///
/// # Returns
/// Vec of SimpleLeaderboardEntry sorted by rank (1 = best, highest metric value)
pub fn calculate_simple_ranking(
    player_stats: Vec<(ChainId, PlayerLifetimeStatistics)>,
    metric: SimpleRankingMetric,
    limit: usize,
) -> Vec<SimpleLeaderboardEntry> {
    // Calculate all metric values for all players
    let mut entries: Vec<(ChainId, PlayerLifetimeStatistics, u128)> = player_stats
        .into_iter()
        .map(|(chain_id, stats)| {
            let primary_metric_value = metric.calculate_value(&stats);
            (chain_id, stats, primary_metric_value)
        })
        .collect();

    // Sort by primary metric value (descending - higher is better)
    entries.sort_by(|a, b| b.2.cmp(&a.2));

    // Apply limit if > 0
    if limit > 0 && entries.len() > limit {
        entries.truncate(limit);
    }

    // Convert to leaderboard entries with rank and all calculated metrics
    entries
        .into_iter()
        .enumerate()
        .map(|(idx, (player_id, stats, _metric_value))| {
            let (net_profit_amount, is_profit) = calculate_net_profit_tuple(&stats);

            SimpleLeaderboardEntry {
                player_id: Some(player_id),
                player_name: stats.player_name.clone(),
                rank: (idx + 1) as u32,
                metric_type: metric.clone(),
                net_profit_amount: Amount::from_attos(net_profit_amount),
                is_profit,
                total_winnings: stats.total_won,
                win_rate: SimpleRankingMetric::WinRate.calculate_value(&stats) as u64,
                games_played: stats.total_games,
                hands_played: stats.total_hands,
                current_streak: stats.current_win_streak,
                blackjack_count: stats.blackjacks_hit,
            }
        })
        .collect()
}
