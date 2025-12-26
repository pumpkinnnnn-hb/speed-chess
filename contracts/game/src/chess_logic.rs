//! Chess logic utilities for FEN parsing and move validation
//!
//! This module provides core chess functionality without external dependencies.
//! It implements FEN parsing, move application, and game end detection.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub board: [[Option<(Piece, Color)>; 8]; 8],
    pub active_color: Color,
    pub castling: CastlingRights,
    pub en_passant: Option<(usize, usize)>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
}

#[derive(Debug, Clone)]
pub struct CastlingRights {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}

impl Position {
    /// Parse FEN string into Position
    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() < 4 {
            return Err("Invalid FEN: not enough parts".to_string());
        }

        let board = Self::parse_board(parts[0])?;
        let active_color = match parts[1] {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return Err("Invalid active color".to_string()),
        };

        let castling = Self::parse_castling(parts[2])?;
        let en_passant = Self::parse_en_passant(parts[3])?;

        let halfmove_clock = if parts.len() > 4 {
            parts[4].parse().unwrap_or(0)
        } else {
            0
        };

        let fullmove_number = if parts.len() > 5 {
            parts[5].parse().unwrap_or(1)
        } else {
            1
        };

        Ok(Position {
            board,
            active_color,
            castling,
            en_passant,
            halfmove_clock,
            fullmove_number,
        })
    }

    /// Parse board part of FEN
    #[allow(clippy::type_complexity)]
    fn parse_board(board_str: &str) -> Result<[[Option<(Piece, Color)>; 8]; 8], String> {
        let mut board = [[None; 8]; 8];
        let ranks: Vec<&str> = board_str.split('/').collect();

        if ranks.len() != 8 {
            return Err("Invalid FEN: board must have 8 ranks".to_string());
        }

        for (rank_idx, rank_str) in ranks.iter().enumerate() {
            let mut file_idx = 0;
            for ch in rank_str.chars() {
                if file_idx >= 8 {
                    return Err("Invalid FEN: rank too long".to_string());
                }

                if ch.is_ascii_digit() {
                    let empty_squares = ch.to_digit(10).unwrap() as usize;
                    file_idx += empty_squares;
                } else {
                    let (piece, color) = Self::parse_piece(ch)?;
                    board[rank_idx][file_idx] = Some((piece, color));
                    file_idx += 1;
                }
            }

            if file_idx != 8 {
                return Err("Invalid FEN: rank too short".to_string());
            }
        }

        Ok(board)
    }

    /// Parse piece character
    fn parse_piece(ch: char) -> Result<(Piece, Color), String> {
        let color = if ch.is_uppercase() {
            Color::White
        } else {
            Color::Black
        };

        let piece = match ch.to_ascii_lowercase() {
            'p' => Piece::Pawn,
            'n' => Piece::Knight,
            'b' => Piece::Bishop,
            'r' => Piece::Rook,
            'q' => Piece::Queen,
            'k' => Piece::King,
            _ => return Err(format!("Invalid piece: {}", ch)),
        };

        Ok((piece, color))
    }

    /// Parse castling rights
    fn parse_castling(castling_str: &str) -> Result<CastlingRights, String> {
        if castling_str == "-" {
            return Ok(CastlingRights {
                white_kingside: false,
                white_queenside: false,
                black_kingside: false,
                black_queenside: false,
            });
        }

        Ok(CastlingRights {
            white_kingside: castling_str.contains('K'),
            white_queenside: castling_str.contains('Q'),
            black_kingside: castling_str.contains('k'),
            black_queenside: castling_str.contains('q'),
        })
    }

    /// Parse en passant square
    fn parse_en_passant(ep_str: &str) -> Result<Option<(usize, usize)>, String> {
        if ep_str == "-" {
            return Ok(None);
        }

        if ep_str.len() != 2 {
            return Err("Invalid en passant square".to_string());
        }

        let file = (ep_str.chars().nth(0).unwrap() as u8 - b'a') as usize;
        let rank = 8 - (ep_str.chars().nth(1).unwrap().to_digit(10).unwrap() as usize);

        Ok(Some((rank, file)))
    }

    /// Convert position to FEN string
    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        // Board
        for rank in &self.board {
            let mut empty_count = 0;
            for square in rank {
                match square {
                    None => empty_count += 1,
                    Some((piece, color)) => {
                        if empty_count > 0 {
                            fen.push_str(&empty_count.to_string());
                            empty_count = 0;
                        }
                        fen.push(Self::piece_to_char(*piece, *color));
                    }
                }
            }
            if empty_count > 0 {
                fen.push_str(&empty_count.to_string());
            }
            fen.push('/');
        }
        fen.pop(); // Remove trailing '/'

        // Active color
        fen.push(' ');
        fen.push(match self.active_color {
            Color::White => 'w',
            Color::Black => 'b',
        });

        // Castling
        fen.push(' ');
        let mut castling = String::new();
        if self.castling.white_kingside {
            castling.push('K');
        }
        if self.castling.white_queenside {
            castling.push('Q');
        }
        if self.castling.black_kingside {
            castling.push('k');
        }
        if self.castling.black_queenside {
            castling.push('q');
        }
        fen.push_str(if castling.is_empty() { "-" } else { &castling });

        // En passant
        fen.push(' ');
        match self.en_passant {
            None => fen.push('-'),
            Some((rank, file)) => {
                fen.push((b'a' + file as u8) as char);
                fen.push_str(&(8 - rank).to_string());
            }
        }

        // Halfmove and fullmove
        fen.push(' ');
        fen.push_str(&self.halfmove_clock.to_string());
        fen.push(' ');
        fen.push_str(&self.fullmove_number.to_string());

        fen
    }

    fn piece_to_char(piece: Piece, color: Color) -> char {
        let ch = match piece {
            Piece::Pawn => 'p',
            Piece::Knight => 'n',
            Piece::Bishop => 'b',
            Piece::Rook => 'r',
            Piece::Queen => 'q',
            Piece::King => 'k',
        };

        match color {
            Color::White => ch.to_ascii_uppercase(),
            Color::Black => ch,
        }
    }

    /// Get piece at square (algebraic notation like "e2")
    pub fn get_piece_at(&self, square: &str) -> Option<String> {
        let (rank, file) = Self::parse_square(square).ok()?;
        self.board[rank][file].map(|(piece, color)| {
            Self::piece_to_char(piece, color).to_string()
        })
    }

    /// Parse algebraic square to indices
    fn parse_square(square: &str) -> Result<(usize, usize), String> {
        if square.len() != 2 {
            return Err("Invalid square".to_string());
        }

        let file = (square.chars().nth(0).unwrap() as u8 - b'a') as usize;
        let rank = 8 - (square.chars().nth(1).unwrap().to_digit(10).ok_or("Invalid rank")? as usize);

        if file >= 8 || rank >= 8 {
            return Err("Square out of bounds".to_string());
        }

        Ok((rank, file))
    }

    /// Validate if a move is legal according to chess rules
    fn is_legal_move(&self, from_rank: usize, from_file: usize, to_rank: usize, to_file: usize, piece: Piece, color: Color) -> bool {
        // Can't move to same square
        if from_rank == to_rank && from_file == to_file {
            return false;
        }

        // Can't capture own piece
        if let Some((_, target_color)) = self.board[to_rank][to_file] {
            if target_color == color {
                return false;
            }
        }

        let rank_diff = (to_rank as i32 - from_rank as i32).abs();
        let file_diff = (to_file as i32 - from_file as i32).abs();

        match piece {
            Piece::Pawn => {
                let direction = if color == Color::White { -1 } else { 1 };
                let expected_rank = (from_rank as i32 + direction) as usize;
                let start_rank = if color == Color::White { 6 } else { 1 };

                // Forward move
                if from_file == to_file && self.board[to_rank][to_file].is_none() {
                    if to_rank == expected_rank {
                        return true; // One square forward
                    }
                    // Two squares from start
                    if from_rank == start_rank && to_rank == (from_rank as i32 + 2 * direction) as usize {
                        let middle_rank = (from_rank as i32 + direction) as usize;
                        return self.board[middle_rank][from_file].is_none();
                    }
                }

                // Diagonal capture
                if file_diff == 1 && to_rank == expected_rank {
                    return self.board[to_rank][to_file].is_some();
                }

                false
            }
            Piece::Knight => {
                (rank_diff == 2 && file_diff == 1) || (rank_diff == 1 && file_diff == 2)
            }
            Piece::Bishop => {
                if rank_diff != file_diff {
                    return false;
                }
                self.is_path_clear(from_rank, from_file, to_rank, to_file)
            }
            Piece::Rook => {
                if from_rank != to_rank && from_file != to_file {
                    return false;
                }
                self.is_path_clear(from_rank, from_file, to_rank, to_file)
            }
            Piece::Queen => {
                let is_diagonal = rank_diff == file_diff;
                let is_straight = from_rank == to_rank || from_file == to_file;
                if !is_diagonal && !is_straight {
                    return false;
                }
                self.is_path_clear(from_rank, from_file, to_rank, to_file)
            }
            Piece::King => {
                rank_diff <= 1 && file_diff <= 1
            }
        }
    }

    /// Check if path between two squares is clear (no pieces blocking)
    fn is_path_clear(&self, from_rank: usize, from_file: usize, to_rank: usize, to_file: usize) -> bool {
        let rank_step = (to_rank as i32 - from_rank as i32).signum();
        let file_step = (to_file as i32 - from_file as i32).signum();

        let mut rank = from_rank as i32 + rank_step;
        let mut file = from_file as i32 + file_step;

        while rank != to_rank as i32 || file != to_file as i32 {
            if self.board[rank as usize][file as usize].is_some() {
                return false;
            }
            rank += rank_step;
            file += file_step;
        }

        true
    }

    /// Apply a move and return new FEN
    pub fn apply_move(&mut self, from: &str, to: &str, promotion: Option<&str>) -> Result<(), String> {
        let (from_rank, from_file) = Self::parse_square(from)?;
        let (to_rank, to_file) = Self::parse_square(to)?;

        // Get piece at source
        let piece_data = self.board[from_rank][from_file]
            .ok_or("No piece at source square")?;

        // Verify it's the active player's piece
        if piece_data.1 != self.active_color {
            return Err("Not your piece".to_string());
        }

        // Validate move is legal
        if !self.is_legal_move(from_rank, from_file, to_rank, to_file, piece_data.0, piece_data.1) {
            return Err(format!("Illegal move: {} cannot move from {} to {}",
                match piece_data.0 {
                    Piece::Pawn => "pawn",
                    Piece::Knight => "knight",
                    Piece::Bishop => "bishop",
                    Piece::Rook => "rook",
                    Piece::Queen => "queen",
                    Piece::King => "king",
                },
                from, to
            ));
        }

        // Check if destination has a piece (capture) BEFORE moving
        let is_capture = self.board[to_rank][to_file].is_some();

        // Move piece
        self.board[to_rank][to_file] = Some(piece_data);
        self.board[from_rank][from_file] = None;

        // Handle promotion
        if let Some(promo) = promotion {
            if piece_data.0 == Piece::Pawn {
                let promoted_piece = match promo {
                    "q" => Piece::Queen,
                    "r" => Piece::Rook,
                    "b" => Piece::Bishop,
                    "n" => Piece::Knight,
                    _ => return Err("Invalid promotion piece".to_string()),
                };
                self.board[to_rank][to_file] = Some((promoted_piece, piece_data.1));
            }
        }

        // Update castling rights (simplified)
        if piece_data.0 == Piece::King {
            match piece_data.1 {
                Color::White => {
                    self.castling.white_kingside = false;
                    self.castling.white_queenside = false;
                }
                Color::Black => {
                    self.castling.black_kingside = false;
                    self.castling.black_queenside = false;
                }
            }
        }

        // Update en passant (simplified - only for pawn double moves)
        self.en_passant = None;
        if piece_data.0 == Piece::Pawn {
            let rank_diff = (to_rank as i32 - from_rank as i32).abs();
            if rank_diff == 2 {
                let ep_rank = (from_rank + to_rank) / 2;
                self.en_passant = Some((ep_rank, from_file));
            }
        }

        // Update clocks - check if pawn move or capture
        if piece_data.0 == Piece::Pawn || is_capture {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        // Switch active color
        if self.active_color == Color::Black {
            self.fullmove_number += 1;
        }
        self.active_color = match self.active_color {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };

        Ok(())
    }

    /// Check if the current position is checkmate or stalemate
    pub fn check_game_end(&self) -> (bool, bool) {
        // Simplified game end detection
        // In a full implementation, this would check for checkmate/stalemate
        // For now, we check for insufficient material or 50-move rule

        let is_stalemate = self.halfmove_clock >= 100; // 50-move rule
        let is_checkmate = false; // Requires full move generation

        (is_checkmate, is_stalemate)
    }

    /// Convert move to Standard Algebraic Notation (simplified)
    pub fn to_san(&self, from: &str, to: &str) -> String {
        // Simplified SAN - just return the move in algebraic notation
        // Full SAN would include piece letter, captures, checks, etc.

        if let Ok((from_rank, from_file)) = Self::parse_square(from) {
            if let Some((piece, _color)) = self.board[from_rank][from_file] {
                let piece_char = match piece {
                    Piece::Pawn => "",
                    Piece::Knight => "N",
                    Piece::Bishop => "B",
                    Piece::Rook => "R",
                    Piece::Queen => "Q",
                    Piece::King => "K",
                };

                // Check if capture
                let capture = if let Ok((to_rank, to_file)) = Self::parse_square(to) {
                    self.board[to_rank][to_file].is_some()
                } else {
                    false
                };

                if piece == Piece::Pawn && capture {
                    return format!("{}x{}", from.chars().nth(0).unwrap(), to);
                } else if capture {
                    return format!("{}x{}", piece_char, to);
                } else {
                    return format!("{}{}", piece_char, to);
                }
            }
        }

        // Fallback
        format!("{}{}", from, to)
    }
}
