use async_graphql_derive::SimpleObject;
use linera_sdk::linera_base_types::{Amount, ChainId};
use serde::{Deserialize, Serialize};

use crate::blackjack::GameOutcome;
use crate::deck::calculate_hand_value;

#[derive(Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize, SimpleObject)]
pub struct Chip {
    pub amount: Amount,
    pub text: String,
    pub enable: bool,
}

#[derive(Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize, SimpleObject)]
pub struct BetData {
    pub min_bet: Amount,
    pub max_bet: Amount,
    pub chipset: Option<[Chip; 5]>,
}

#[derive(Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize, SimpleObject)]
pub struct Profile {
    pub seat: Option<u8>,
    pub balance: Amount,
    pub my_chain: Option<ChainId>,
    pub bet_data: Option<BetData>,
    pub statistics: PlayerLifetimeStatistics,
}

impl Profile {
    pub fn update_seat(&mut self, seat_id: u8) {
        self.seat = Some(seat_id)
    }

    pub fn remove_seat(&mut self) {
        self.seat = None
    }

    pub fn update_balance(&mut self, amount: Amount) {
        self.balance = amount
    }

    pub fn update_chain_id(&mut self, current_chain: ChainId) {
        self.my_chain = Some(current_chain)
    }

    pub fn calculate_bet_data(&mut self) {
        // Minimum base value (smallest chip)
        let mut base = Amount::from_tokens(100);

        // Handle balances below the minimum
        if self.balance < base {
            self.bet_data = Some(BetData {
                min_bet: Amount::ZERO,
                max_bet: self.balance,
                chipset: None,
            });
            return;
        }

        // Calculate the appropriate base level
        while self.balance >= base.try_mul(500).unwrap_or(Amount::MAX) {
            let next_base = base.try_mul(10).unwrap_or(Amount::ZERO);
            if next_base.is_zero() {
                break;
            } else {
                base = next_base;
            }
        }

        // Generate chip denominations
        let denominations = [
            base,                     // 1x
            base.saturating_mul(5),   // 5x
            base.saturating_mul(25),  // 25x
            base.saturating_mul(100), // 100x
            base.saturating_mul(250), // 250x
        ];

        // Generate chip list
        let generated_chip_list = [
            Chip {
                amount: denominations[0],
                text: format_chip_units(denominations[0].to_attos()),
                enable: denominations[0] <= self.balance,
            },
            Chip {
                amount: denominations[1],
                text: format_chip_units(denominations[1].to_attos()),
                enable: denominations[1] <= self.balance,
            },
            Chip {
                amount: denominations[2],
                text: format_chip_units(denominations[2].to_attos()),
                enable: denominations[2] <= self.balance,
            },
            Chip {
                amount: denominations[3],
                text: format_chip_units(denominations[3].to_attos()),
                enable: denominations[3] <= self.balance,
            },
            Chip {
                amount: denominations[4],
                text: format_chip_units(denominations[4].to_attos()),
                enable: denominations[4] <= self.balance,
            },
        ];

        self.bet_data = Some(BetData {
            min_bet: denominations[0], // Smallest denomination
            max_bet: self.balance,     // Player's full balance
            chipset: Some(generated_chip_list),
        })
    }

    pub fn clear_bet_data(&mut self) {
        self.bet_data = None
    }

    /// Update player statistics after a hand completes
    pub fn update_statistics_from_hand(
        &mut self,
        player_name: &Option<String>,
        player_hand: &Vec<u8>,
        player_outcome: &GameOutcome,
        player_bet: Amount,
        dealer_hand: &Vec<u8>,
    ) {
        self.statistics
            .update_from_hand(player_name, player_hand, player_outcome, player_bet, dealer_hand);
    }
}

/// Check if a hand is a blackjack (21 with exactly 2 cards)
fn is_blackjack(hand: &Vec<u8>) -> bool {
    hand.len() == 2 && calculate_hand_value(hand) == 21
}

/// Check if a hand is busted (value > 21)
fn is_bust(hand: &Vec<u8>) -> bool {
    calculate_hand_value(hand) > 21
}

/// Check if a hand is a five-card charlie (5 cards without busting)
fn is_five_card_charlie(hand: &Vec<u8>) -> bool {
    hand.len() == 5 && calculate_hand_value(hand) <= 21
}

/// Check if a hand is a perfect 21 (21 with more than 2 cards, not blackjack)
fn is_perfect_21(hand: &Vec<u8>) -> bool {
    hand.len() > 2 && calculate_hand_value(hand) == 21
}

#[derive(Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize, SimpleObject)]
pub struct PlayerLifetimeStatistics {
    // Profile
    pub player_name: String,

    // Game Counts
    pub total_games: u64,
    pub total_hands: u64,

    // Win/Loss
    pub hands_won: u64,
    pub hands_lost: u64,
    pub hands_pushed: u64,
    pub games_won: u64,
    pub games_lost: u64,

    // Financial
    pub total_wagered: Amount,
    pub total_won: Amount,
    pub total_lost: Amount,
    pub biggest_single_win: Amount,
    pub biggest_single_loss: Amount,

    // Blackjack Specific
    pub blackjacks_hit: u64,
    pub busts: u64,
    pub dealer_busts_witnessed: u64,
    pub perfect_hands: u64,     // 21 without blackjack
    pub five_card_charlie: u64, // 5 cards without busting

    // Streaks
    pub current_win_streak: u64,
    pub current_loss_streak: u64,
    pub best_win_streak: u64,
    pub worst_loss_streak: u64,
}

impl PlayerLifetimeStatistics {
    /// Update statistics after a hand completes
    ///
    /// # Parameters
    /// - `player_hand`: The player's final hand
    /// - `player_outcome`: The game outcome (PlayerWins/DealerWins/Draw)
    /// - `player_bet`: The amount wagered
    /// - `dealer_hand`: The dealer's final hand (to detect dealer bust)
    pub fn update_from_hand(
        &mut self,
        player_name: &Option<String>,
        player_hand: &Vec<u8>,
        player_outcome: &GameOutcome,
        player_bet: Amount,
        dealer_hand: &Vec<u8>,
    ) {
        self.player_name = player_name.clone().expect("Player name is None for PlayerLifetimeStatistics update_from_hand");

        // Always increment total hands and total games (same thing)
        self.total_hands += 1;
        self.total_games += 1;

        // Always track wagered amount
        self.total_wagered = self.total_wagered.saturating_add(player_bet);

        // Detect special hand types
        if is_blackjack(player_hand) {
            self.blackjacks_hit += 1;
        }

        if is_bust(player_hand) {
            self.busts += 1;
        }

        if is_five_card_charlie(player_hand) {
            self.five_card_charlie += 1;
        }

        if is_perfect_21(player_hand) {
            self.perfect_hands += 1;
        }

        // Check if dealer busted (only if dealer played)
        let dealer_busted = is_bust(dealer_hand);
        if dealer_busted {
            self.dealer_busts_witnessed += 1;
        }

        // Update win/loss/push statistics and financials
        match player_outcome {
            GameOutcome::PlayerWins => {
                self.hands_won += 1;
                self.games_won += 1; // Same as hands_won
                self.total_won = self.total_won.saturating_add(player_bet);

                // Update win streak
                self.current_win_streak += 1;
                self.current_loss_streak = 0;
                if self.current_win_streak > self.best_win_streak {
                    self.best_win_streak = self.current_win_streak;
                }

                // Track the biggest single win
                if player_bet > self.biggest_single_win {
                    self.biggest_single_win = player_bet;
                }
            }
            GameOutcome::DealerWins => {
                self.hands_lost += 1;
                self.games_lost += 1; // Same as hands_lost
                self.total_lost = self.total_lost.saturating_add(player_bet);

                // Update loss streak
                self.current_loss_streak += 1;
                self.current_win_streak = 0;
                if self.current_loss_streak > self.worst_loss_streak {
                    self.worst_loss_streak = self.current_loss_streak;
                }

                // Track the biggest single loss
                if player_bet > self.biggest_single_loss {
                    self.biggest_single_loss = player_bet;
                }
            }
            GameOutcome::Draw => {
                self.hands_pushed += 1;
                // Reset both streaks on push
                self.current_win_streak = 0;
                self.current_loss_streak = 0;
            }
            GameOutcome::None => {
                // Should not happen - outcome should be determined
                // Do nothing
            }
        }
    }
}

pub fn format_chip_units(value: u128) -> String {
    // Convert from attos to tokens (1 token = 10^18 attos)
    const ATTOS_PER_TOKEN: u128 = 1_000_000_000_000_000_000;
    let value = value / ATTOS_PER_TOKEN;

    if value < 1000 {
        return value.to_string();
    }

    const SUFFIXES: [(&str, u128); 11] = [
        ("D", 1_000_000_000_000_000_000_000_000_000_000_000), // 10^33
        ("N", 1_000_000_000_000_000_000_000_000_000_000),     // 10^30
        ("O", 1_000_000_000_000_000_000_000_000_000),         // 10^27
        ("Sp", 1_000_000_000_000_000_000_000_000),            // 10^24
        ("S", 1_000_000_000_000_000_000_000),                 // 10^21
        ("Qi", 1_000_000_000_000_000_000),                    // 10^18
        ("Q", 1_000_000_000_000_000),                         // 10^15
        ("T", 1_000_000_000_000),                             // 10^12
        ("B", 1_000_000_000),                                 // 10^9
        ("M", 1_000_000),                                     // 10^6
        ("K", 1_000),                                         // 10^3
    ];

    for &(suffix, divisor) in SUFFIXES.iter() {
        if value >= divisor {
            let scaled = value as f64 / divisor as f64;
            // Handle values that round up to 1000
            if scaled >= 999.95 {
                // Try next higher suffix
                if let Some(&(next_suffix, next_divisor)) = SUFFIXES.get(SUFFIXES.len() - SUFFIXES.iter().position(|&s| s.1 == divisor).unwrap() - 1) {
                    let next_scaled = value as f64 / next_divisor as f64;
                    return format_chip_float(next_scaled, next_suffix);
                }
            }
            return format_chip_float(scaled, suffix);
        }
    }

    value.to_string()
}

fn format_chip_float(value: f64, suffix: &str) -> String {
    // Round to the nearest tenth
    let rounded = (value * 10.0).round() / 10.0;

    if rounded.fract() == 0.0 {
        format!("{:.0}{}", rounded, suffix)
    } else {
        let s = format!("{:.1}", rounded);
        s.trim_end_matches('0').trim_end_matches('.').to_string() + suffix
    }
}
