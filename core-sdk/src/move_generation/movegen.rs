use super::magic::{self};
use crate::bitboards::bitboards;
use crate::bitboards::bitboards::constants::{
    square, BISHOP_RAYS, FREEFIELD_BISHOP_ATTACKS, FREEFIELD_ROOK_ATTACKS, KING_ATTACKS,
    KNIGHT_ATTACKS, RANKS, ROOK_RAYS,
};
use crate::bitboards::bitboards::square;
use crate::board_representation::game_state::{
    GameMove, GameMoveType, GameState, PieceType, WHITE,
};
use crate::search::GradedMove;

impl GameState {
    //Attacks from rook + queen
    //Rook+Queen attacks xray the king of the color to move, if side != color_to_move
    pub fn get_major_attacks_from_side(&self, side: usize) -> u64 {
        let occupied_squares = if side == self.get_color_to_move() {
            self.get_all_pieces()
        } else {
            self.get_all_pieces_without_ctm_king()
        };
        let mut res = 0u64;
        //TODO Kogge Stone
        for pt in [PieceType::Rook, PieceType::Queen].iter() {
            let mut piece = self.get_piece(*pt, side);
            while piece > 0 {
                let idx = piece.trailing_zeros() as usize;
                res |= (*pt).attacks(idx, occupied_squares);
                piece ^= square(idx);
            }
        }
        res
    }
    //Attacks from bishop + knight + pawns
    //Bishop attacks xray the king of the color to move, if side != color_to_move
    pub fn get_minor_attacks_from_side(&self, side: usize) -> u64 {
        let occupied_squares = if side == self.get_color_to_move() {
            self.get_all_pieces()
        } else {
            self.get_all_pieces_without_ctm_king()
        };
        let mut res = 0u64;
        //TODO Kogge Stone
        for pt in [PieceType::Knight, PieceType::Bishop].iter() {
            let mut piece = self.get_piece(*pt, side);
            while piece > 0 {
                let idx = piece.trailing_zeros() as usize;
                res |= (*pt).attacks(idx, occupied_squares);
                piece ^= square(idx);
            }
        }
        res |= pawn_targets(side, self.get_piece(PieceType::Pawn, side));
        res
    }

    //King + major + minor attacks from side
    pub fn get_attacks_from_side(&self, side: usize) -> u64 {
        self.get_major_attacks_from_side(side)
            | self.get_minor_attacks_from_side(side)
            | PieceType::King.attacks(self.get_king_square(side), 0u64)
    }

    //Returns true if the given square is attacked by the side not to move
    //occ: Blockers in the current position. Might be all_pieces or all_pieces without ctm king
    //exclude: Exclude all attacks from pieces given in the exclude bitboard
    pub fn square_attacked(&self, sq: usize, occ: u64, exclude: u64) -> bool {
        let square = square(sq);
        PieceType::King.attacks(sq, occ)
            & self.get_piece(PieceType::King, 1 - self.get_color_to_move())
            & !exclude
            > 0
            || PieceType::Knight.attacks(sq, occ)
                & self.get_piece(PieceType::Knight, 1 - self.get_color_to_move())
                & !exclude
                > 0
            || PieceType::Bishop.attacks(sq, occ)
                & self.get_bishop_like_bb(1 - self.get_color_to_move())
                & !exclude
                > 0
            || PieceType::Rook.attacks(sq, occ)
                & self.get_rook_like_bb(1 - self.get_color_to_move())
                & !exclude
                > 0
            || pawn_targets(self.get_color_to_move(), square)
                & self.get_piece(PieceType::Pawn, 1 - self.get_color_to_move())
                & !exclude
                > 0
    }
    //Returns a bitboard of allthe pieces attacking the square
    //Occ: Blockers in the current position. Might be all_pieces or all_pieces without ctm king
    pub fn square_attackers(&self, sq: usize, occ: u64) -> u64 {
        let square = square(sq);
        PieceType::King.attacks(sq, occ)
            & self.get_piece(PieceType::King, 1 - self.get_color_to_move())
            | PieceType::Knight.attacks(sq, occ)
                & self.get_piece(PieceType::Knight, 1 - self.get_color_to_move())
            | PieceType::Bishop.attacks(sq, occ)
                & self.get_bishop_like_bb(1 - self.get_color_to_move())
            | PieceType::Rook.attacks(sq, occ) & self.get_rook_like_bb(1 - self.get_color_to_move())
            | pawn_targets(self.get_color_to_move(), square)
                & self.get_piece(PieceType::Pawn, 1 - self.get_color_to_move())
    }

    pub fn get_checkers(&self) -> u64 {
        self.square_attackers(
            self.get_king_square(self.get_color_to_move()),
            self.get_all_pieces(),
        )
    }
    pub fn in_check(&self) -> bool {
        self.square_attacked(
            self.get_king_square(self.get_color_to_move()),
            self.get_all_pieces(),
            0u64,
        )
    }
}
impl PieceType {
    //Occ not needed for PieceType::King, PieceType::Knight
    #[inline(always)]
    pub fn attacks(&self, from: usize, occ: u64) -> u64 {
        match self {
            PieceType::King => KING_ATTACKS[from],
            PieceType::Knight => KNIGHT_ATTACKS[from],
            PieceType::Rook => rook_attack(from, occ),
            PieceType::Queen => bishop_attack(from, occ) | rook_attack(from, occ),
            PieceType::Bishop => bishop_attack(from, occ),
            _ => panic!("Pawn is not supported due to branching!"),
        }
    }
}
#[inline(always)]
pub fn bishop_attack(square: usize, all_pieces: u64) -> u64 {
    magic::Magic::bishop(square, all_pieces)
}

#[inline(always)]
pub fn rook_attack(square: usize, all_pieces: u64) -> u64 {
    magic::Magic::rook(square, all_pieces)
}

//Pawn single pushes

#[inline(always)]
pub fn single_push_pawn_targets(side: usize, pawns: u64, empty: u64) -> u64 {
    if side == WHITE {
        w_single_push_pawn_targets(pawns, empty)
    } else {
        b_single_push_pawn_targets(pawns, empty)
    }
}

#[inline(always)]
pub fn w_single_push_pawn_targets(pawns: u64, empty: u64) -> u64 {
    bitboards::north_one(pawns) & empty
}

#[inline(always)]
pub fn b_single_push_pawn_targets(pawns: u64, empty: u64) -> u64 {
    bitboards::south_one(pawns) & empty
}

//Pawn double pushes
#[inline(always)]
pub fn double_push_pawn_targets(side: usize, pawns: u64, empty: u64) -> u64 {
    if side == WHITE {
        w_double_push_pawn_targets(pawns, empty)
    } else {
        b_double_push_pawn_targets(pawns, empty)
    }
}

#[inline(always)]
pub fn w_double_push_pawn_targets(pawns: u64, empty: u64) -> u64 {
    bitboards::north_one(bitboards::north_one(pawns & RANKS[1]) & empty) & empty
}

#[inline(always)]
pub fn b_double_push_pawn_targets(pawns: u64, empty: u64) -> u64 {
    bitboards::south_one(bitboards::south_one(pawns & RANKS[6]) & empty) & empty
}

#[inline(always)]
pub fn pawn_targets(side: usize, pawns: u64) -> u64 {
    pawn_east_targets(side, pawns) | pawn_west_targets(side, pawns)
}
//Pawn east targets
#[inline(always)]
pub fn pawn_east_targets(side: usize, pawns: u64) -> u64 {
    if side == WHITE {
        w_pawn_east_targets(pawns)
    } else {
        b_pawn_east_targets(pawns)
    }
}

//NorthEast = +9
#[inline(always)]
pub fn w_pawn_east_targets(pawns: u64) -> u64 {
    bitboards::north_east_one(pawns)
}

//SouthEast = -7
#[inline(always)]
pub fn b_pawn_east_targets(pawns: u64) -> u64 {
    bitboards::south_west_one(pawns)
}

//Pawn west targets
#[inline(always)]
pub fn pawn_west_targets(side: usize, pawns: u64) -> u64 {
    if side == WHITE {
        w_pawn_west_targets(pawns)
    } else {
        b_pawn_west_targets(pawns)
    }
}

//NorthWest = +7
#[inline(always)]
pub fn w_pawn_west_targets(pawns: u64) -> u64 {
    bitboards::north_west_one(pawns)
}

//NorthWest = -9
#[inline(always)]
pub fn b_pawn_west_targets(pawns: u64) -> u64 {
    bitboards::south_east_one(pawns)
}

#[inline(always)]
pub fn find_captured_piece_type(g: &GameState, to: usize) -> PieceType {
    let to_board = square(to);
    let side = g.get_color_to_move();
    if g.get_piece(PieceType::Pawn, 1 - side) & to_board != 0u64 {
        PieceType::Pawn
    } else if g.get_piece(PieceType::Knight, 1 - side) & to_board != 0u64 {
        PieceType::Knight
    } else if g.get_piece(PieceType::Queen, 1 - side) & to_board != 0u64 {
        PieceType::Queen
    } else if g.get_piece(PieceType::Bishop, 1 - side) & to_board != 0u64 {
        PieceType::Bishop
    } else if g.get_piece(PieceType::Rook, 1 - side) & to_board != 0u64 {
        PieceType::Rook
    } else {
        panic!("Shoudln't get here");
    }
}

#[inline(always)]
pub fn xray_rook_attacks(
    rook_attacks: u64,
    occupied_squares: u64,
    my_pieces: u64,
    rook_square: usize,
) -> u64 {
    rook_attacks ^ rook_attack(rook_square, occupied_squares ^ (my_pieces & rook_attacks))
}

#[inline(always)]
pub fn xray_bishop_attacks(
    bishop_attacks: u64,
    occupied_squares: u64,
    my_pieces: u64,
    bishop_square: usize,
) -> u64 {
    bishop_attacks
        ^ bishop_attack(
            bishop_square,
            occupied_squares ^ (my_pieces & bishop_attacks),
        )
}

#[inline(always)]
pub fn add_pin_moves_to_movelist(
    legal_moves: &mut MoveList,
    only_captures: bool,
    ray_to_king: u64,
    push_mask: u64,
    capture_mask: u64,
    enemy_pinner: u64,
    pinned_piece_position: usize,
    moving_piece_type: PieceType,
    pinner_position: usize,
    enemy_queens: u64,
    other_pinner_piece_type: PieceType,
) {
    let pin_quiet_targets = ray_to_king & push_mask & !square(pinned_piece_position);
    let pin_capture_possible = (capture_mask & enemy_pinner) != 0u64;
    if !only_captures {
        add_moves_to_movelist(
            legal_moves,
            pinned_piece_position,
            pin_quiet_targets,
            moving_piece_type,
            GameMoveType::Quiet,
        );
    }
    if pin_capture_possible {
        add_move_to_movelist(
            legal_moves,
            pinned_piece_position,
            pinner_position,
            moving_piece_type,
            GameMoveType::Capture(if enemy_pinner & enemy_queens != 0u64 {
                PieceType::Queen
            } else {
                other_pinner_piece_type
            }),
        );
    }
}

#[inline(always)]
pub fn add_king_moves_to_movelist(
    g: &GameState,
    legal_moves: &mut MoveList,
    only_captures: bool,
    stm_legal_kingmoves: u64,
    stm_king_index: usize,
    enemy_pieces: u64,
) {
    let mut captures = stm_legal_kingmoves & enemy_pieces;
    let quiets = stm_legal_kingmoves & !captures;
    while captures != 0u64 {
        let capture_index = captures.trailing_zeros() as usize;
        add_move_to_movelist(
            legal_moves,
            stm_king_index,
            capture_index,
            PieceType::King,
            GameMoveType::Capture(find_captured_piece_type(g, capture_index)),
        );
        captures ^= square(capture_index);
    }
    if !only_captures {
        add_moves_to_movelist(
            legal_moves,
            stm_king_index,
            quiets,
            PieceType::King,
            GameMoveType::Quiet,
        );
    }
}

#[inline(always)]
pub fn add_pawn_moves_to_movelist(
    g: &GameState,
    legal_moves: &mut MoveList,
    mut target_board: u64,
    shift: usize,
    is_capture: bool,
    is_promotion: bool,
    pinned_pieces: u64,
) {
    while target_board != 0u64 {
        let pawn_index = target_board.trailing_zeros() as usize;
        let pawn = square(pawn_index);
        let from_index = if g.get_color_to_move() == WHITE {
            pawn_index - shift
        } else {
            pawn_index + shift
        };
        let from_board = square(from_index);
        if from_board & pinned_pieces == 0u64 {
            let mv_type = if is_capture {
                GameMoveType::Capture(find_captured_piece_type(g, pawn_index))
            } else {
                GameMoveType::Quiet
            };
            if is_promotion {
                add_promotion_move_to_movelist(legal_moves, from_index, pawn_index, mv_type);
            } else {
                add_move_to_movelist(
                    legal_moves,
                    from_index,
                    pawn_index,
                    PieceType::Pawn,
                    mv_type,
                )
            }
        }
        target_board ^= pawn;
    }
}

#[inline(always)]
pub fn add_normal_moves_to_movelist(
    g: &GameState,
    legal_moves: &mut MoveList,
    piece_type: PieceType,
    mut piece_board: u64,
    pinned_pieces: u64,
    enemy_pieces: u64,
    empty_squares: u64,
    push_mask: u64,
    capture_mask: u64,
    only_captures: bool,
) {
    while piece_board != 0u64 {
        let piece_index = piece_board.trailing_zeros() as usize;
        let piece = square(piece_index);
        if pinned_pieces & piece == 0 {
            let piece_target = piece_type.attacks(piece_index, g.get_all_pieces());
            let mut captures = piece_target & capture_mask & enemy_pieces;
            while captures != 0u64 {
                let capture_index = captures.trailing_zeros() as usize;
                add_move_to_movelist(
                    legal_moves,
                    piece_index,
                    capture_index,
                    piece_type,
                    GameMoveType::Capture(find_captured_piece_type(g, capture_index)),
                );
                captures ^= square(capture_index);
            }

            if !only_captures {
                let quiets = piece_target & push_mask & empty_squares;
                add_moves_to_movelist(
                    legal_moves,
                    piece_index,
                    quiets,
                    piece_type,
                    GameMoveType::Quiet,
                );
            }
        }
        piece_board ^= piece;
    }
}

#[inline(always)]
pub fn add_promotion_move_to_movelist(
    legal_moves: &mut MoveList,
    from_square: usize,
    to_square: usize,
    move_type: GameMoveType,
) {
    let new_types = if let GameMoveType::Capture(x) = move_type {
        (
            GameMoveType::Promotion(PieceType::Queen, Some(x)),
            GameMoveType::Promotion(PieceType::Rook, Some(x)),
            GameMoveType::Promotion(PieceType::Bishop, Some(x)),
            GameMoveType::Promotion(PieceType::Knight, Some(x)),
        )
    } else {
        (
            GameMoveType::Promotion(PieceType::Queen, None),
            GameMoveType::Promotion(PieceType::Rook, None),
            GameMoveType::Promotion(PieceType::Bishop, None),
            GameMoveType::Promotion(PieceType::Knight, None),
        )
    };
    add_move_to_movelist(
        legal_moves,
        from_square,
        to_square,
        PieceType::Pawn,
        new_types.0,
    );
    add_move_to_movelist(
        legal_moves,
        from_square,
        to_square,
        PieceType::Pawn,
        new_types.1,
    );
    add_move_to_movelist(
        legal_moves,
        from_square,
        to_square,
        PieceType::Pawn,
        new_types.2,
    );
    add_move_to_movelist(
        legal_moves,
        from_square,
        to_square,
        PieceType::Pawn,
        new_types.3,
    );
}

#[inline(always)]
pub fn add_moves_to_movelist(
    legal_moves: &mut MoveList,
    from_square: usize,
    mut target_board: u64,
    piece_type: PieceType,
    move_type: GameMoveType,
) {
    while target_board != 0u64 {
        let target_square = target_board.trailing_zeros() as usize;
        add_move_to_movelist(
            legal_moves,
            from_square,
            target_square,
            piece_type,
            move_type,
        );
        target_board ^= square(target_square);
    }
}

#[inline(always)]
pub fn add_move_to_movelist(
    legal_moves: &mut MoveList,
    from_square: usize,
    to_square: usize,
    piece_type: PieceType,
    move_type: GameMoveType,
) {
    legal_moves.add_move(GameMove {
        from: from_square as u8,
        to: to_square as u8,
        move_type,
        piece_type,
    });
}

#[derive(Clone)]
pub struct AdditionalGameStateInformation {
    pub stm_incheck: bool,
}

pub const MAX_MOVES: usize = 128;

pub struct MoveList {
    pub move_list: Vec<GradedMove>,
}
impl Default for MoveList {
    fn default() -> Self {
        let move_list = Vec::with_capacity(MAX_MOVES);
        MoveList { move_list }
    }
}

impl MoveList {
    #[inline(always)]
    pub fn add_move(&mut self, mv: GameMove) {
        self.move_list.push(GradedMove(mv, None));
    }

    #[inline(always)]
    pub fn find_move(&self, mv: GameMove, contains: bool) -> usize {
        for (index, mvs) in self.move_list.iter().enumerate() {
            if mvs.0 == mv {
                return index;
            }
        }
        if contains {
            panic!("Type 2 error")
        }
        self.move_list.len()
    }

    #[inline(always)]
    pub fn highest_score(&mut self) -> Option<(usize, GradedMove)> {
        let mut best_index = self.move_list.len();
        let mut best_score = -1_000_000_000.;
        for (index, gmv) in self.move_list.iter().enumerate() {
            if gmv.1.is_some() && gmv.1.unwrap() > best_score {
                best_index = index;
                best_score = gmv.1.unwrap();
            }
        }
        if best_index == self.move_list.len() {
            None
        } else {
            Some((best_index, self.move_list[best_index]))
        }
    }
}

pub fn generate_moves(
    g: &GameState,
    only_captures: bool,
    movelist: &mut MoveList,
) -> AdditionalGameStateInformation {
    //----------------------------------------------------------------------
    //**********************************************************************
    //1. General bitboards and variable initialization
    movelist.move_list.clear();

    let side = g.get_color_to_move();
    let enemy = 1 - side;
    let stm_color_iswhite: bool = side == WHITE;

    let mut side_pawns = g.get_piece(PieceType::Pawn, side);
    let side_pieces = g.get_pieces_from_side(side);
    let enemy_pieces = g.get_pieces_from_side(enemy);
    let all_pieces = enemy_pieces | side_pieces;
    let empty_squares = !all_pieces;

    let enemy_attacks = g.get_attacks_from_side(enemy); //TODO: Check if square_attacked is faster

    //----------------------------------------------------------------------
    //**********************************************************************
    //2. Safe King moves
    let stm_legal_kingmoves = PieceType::King.attacks(g.get_king_square(side), all_pieces)
        & !enemy_attacks
        & !side_pieces;
    add_king_moves_to_movelist(
        g,
        movelist,
        only_captures,
        stm_legal_kingmoves,
        g.get_king_square(side),
        enemy_pieces,
    );
    //----------------------------------------------------------------------
    //**********************************************************************
    //3. Check & Check Evasions
    let check_board = g.get_checkers();
    let checkers = check_board.count_ones() as usize;
    let stm_incheck = checkers > 0;

    let mut capture_mask = 0xFFFF_FFFF_FFFF_FFFFu64;
    let mut push_mask = 0xFFFF_FFFF_FFFF_FFFFu64;
    if checkers > 1 {
        //Double check, only safe king moves are legal
        return AdditionalGameStateInformation { stm_incheck };
    } else if checkers == 1 {
        //Only a single checker
        capture_mask = check_board;
        //If it's a slider, we can also push in its way
        if check_board & (g.get_bishop_like_bb(enemy) | g.get_piece(PieceType::Rook, enemy)) != 0u64
        {
            let checker_square = check_board.trailing_zeros() as usize;
            if check_board & (FREEFIELD_ROOK_ATTACKS[g.get_king_square(side)]) != 0u64 {
                //Checker is rook-like
                push_mask = ROOK_RAYS[g.get_king_square(side)][checker_square];
            } else {
                //Checker is bishop-like
                push_mask = BISHOP_RAYS[g.get_king_square(side)][checker_square];
            }
        } else {
            //else, we can't do push (quiet) moves
            push_mask = 0u64;
        }
    }

    //----------------------------------------------------------------------
    //**********************************************************************
    //4. Pins and pinned pieces
    let mut pinned_pieces = 0u64;
    //4.1 Rook-Like pins
    if FREEFIELD_ROOK_ATTACKS[g.get_king_square(side)] & g.get_rook_like_bb(enemy) != 0u64 {
        let stm_rook_attacks_from_king = rook_attack(g.get_king_square(side), all_pieces);
        let stm_xray_rook_attacks_from_king = xray_rook_attacks(
            stm_rook_attacks_from_king,
            all_pieces,
            side_pieces,
            g.get_king_square(side),
        );
        let mut enemy_rooks_on_xray = stm_xray_rook_attacks_from_king & g.get_rook_like_bb(enemy);
        while enemy_rooks_on_xray != 0u64 {
            let enemy_rook_position = enemy_rooks_on_xray.trailing_zeros() as usize;
            let enemy_rook = square(enemy_rook_position);
            let ray_to_king = ROOK_RAYS[g.get_king_square(side)][enemy_rook_position];
            let pinned_piece = ray_to_king & side_pieces;
            let pinned_piece_position = pinned_piece.trailing_zeros() as usize;
            pinned_pieces |= pinned_piece;
            if pinned_piece & g.get_piece(PieceType::Queen, side) != 0u64 {
                //Add possible queen pushes
                add_pin_moves_to_movelist(
                    movelist,
                    only_captures,
                    ray_to_king,
                    push_mask,
                    capture_mask,
                    enemy_rook,
                    pinned_piece_position,
                    PieceType::Queen,
                    enemy_rook_position,
                    g.get_piece(PieceType::Queen, enemy),
                    PieceType::Rook,
                );
            } else if pinned_piece & g.get_piece(PieceType::Rook, side) != 0u64 {
                //Add possible rook pushes
                add_pin_moves_to_movelist(
                    movelist,
                    only_captures,
                    ray_to_king,
                    push_mask,
                    capture_mask,
                    enemy_rook,
                    pinned_piece_position,
                    PieceType::Rook,
                    enemy_rook_position,
                    g.get_piece(PieceType::Queen, enemy),
                    PieceType::Rook,
                );
            } else if pinned_piece & side_pawns != 0u64 {
                //Add possible pawn pushes
                side_pawns ^= pinned_piece;
                let stm_pawn_pin_single_push = if stm_color_iswhite {
                    w_single_push_pawn_targets(pinned_piece, empty_squares)
                } else {
                    b_single_push_pawn_targets(pinned_piece, empty_squares)
                } & ray_to_king
                    & push_mask;
                let stm_pawn_pin_double_push = if stm_color_iswhite {
                    w_double_push_pawn_targets(pinned_piece, empty_squares)
                } else {
                    b_double_push_pawn_targets(pinned_piece, empty_squares)
                } & ray_to_king
                    & push_mask;
                if !only_captures {
                    add_moves_to_movelist(
                        movelist,
                        pinned_piece_position,
                        stm_pawn_pin_single_push | stm_pawn_pin_double_push,
                        PieceType::Pawn,
                        GameMoveType::Quiet,
                    )
                }
            }
            enemy_rooks_on_xray ^= enemy_rook;
        }
    }
    //4.2 Bishop-Like pins
    if FREEFIELD_BISHOP_ATTACKS[g.get_king_square(side)] & g.get_bishop_like_bb(enemy) != 0u64 {
        let stm_bishop_attacks_from_king = bishop_attack(g.get_king_square(side), all_pieces);
        let stm_xray_bishop_attacks_from_king = xray_bishop_attacks(
            stm_bishop_attacks_from_king,
            all_pieces,
            side_pieces,
            g.get_king_square(side),
        );
        let mut enemy_bishop_on_xray =
            stm_xray_bishop_attacks_from_king & g.get_bishop_like_bb(enemy);
        while enemy_bishop_on_xray != 0u64 {
            let enemy_bishop_position = enemy_bishop_on_xray.trailing_zeros() as usize;
            let enemy_bishop = square(enemy_bishop_position);
            let ray_to_king = BISHOP_RAYS[g.get_king_square(side)][enemy_bishop_position];
            let pinned_piece = ray_to_king & side_pieces;
            let pinned_piece_position = pinned_piece.trailing_zeros() as usize;
            pinned_pieces |= pinned_piece;
            if pinned_piece & g.get_piece(PieceType::Queen, side) != 0u64 {
                //Add possible queen pushes
                add_pin_moves_to_movelist(
                    movelist,
                    only_captures,
                    ray_to_king,
                    push_mask,
                    capture_mask,
                    enemy_bishop,
                    pinned_piece_position,
                    PieceType::Queen,
                    enemy_bishop_position,
                    g.get_piece(PieceType::Queen, enemy),
                    PieceType::Bishop,
                );
            } else if pinned_piece & g.get_piece(PieceType::Bishop, side) != 0u64 {
                //Add possible bishop pushes
                add_pin_moves_to_movelist(
                    movelist,
                    only_captures,
                    ray_to_king,
                    push_mask,
                    capture_mask,
                    enemy_bishop,
                    pinned_piece_position,
                    PieceType::Bishop,
                    enemy_bishop_position,
                    g.get_piece(PieceType::Queen, enemy),
                    PieceType::Bishop,
                );
            } else if pinned_piece & side_pawns != 0u64 {
                //Add possible pawn captures
                side_pawns ^= pinned_piece;

                let stm_pawn_pin_target = if stm_color_iswhite {
                    w_pawn_east_targets(pinned_piece) | w_pawn_west_targets(pinned_piece)
                } else {
                    b_pawn_east_targets(pinned_piece) | b_pawn_west_targets(pinned_piece)
                };
                //Normal captures
                let stm_pawn_pin_captures = stm_pawn_pin_target & capture_mask & enemy_bishop;
                let stm_pawn_pin_promotion_capture =
                    stm_pawn_pin_captures & RANKS[if stm_color_iswhite { 7 } else { 0 }];
                if stm_pawn_pin_promotion_capture != 0u64 {
                    add_promotion_move_to_movelist(
                        movelist,
                        pinned_piece_position,
                        enemy_bishop_position,
                        GameMoveType::Capture(
                            if enemy_bishop & g.get_piece(PieceType::Queen, enemy) != 0u64 {
                                PieceType::Queen
                            } else {
                                PieceType::Bishop
                            },
                        ),
                    );
                }
                let stm_pawn_pin_nonpromotion_capture =
                    stm_pawn_pin_captures & !stm_pawn_pin_promotion_capture;
                if stm_pawn_pin_nonpromotion_capture != 0u64 {
                    add_move_to_movelist(
                        movelist,
                        pinned_piece_position,
                        enemy_bishop_position,
                        PieceType::Pawn,
                        GameMoveType::Capture(
                            if enemy_bishop & g.get_piece(PieceType::Queen, enemy) != 0u64 {
                                PieceType::Queen
                            } else {
                                PieceType::Bishop
                            },
                        ),
                    );
                }
                //En passants
                let stm_pawn_pin_enpassant =
                    stm_pawn_pin_target & g.get_en_passant() & capture_mask & ray_to_king;
                if stm_pawn_pin_enpassant != 0u64 {
                    add_move_to_movelist(
                        movelist,
                        pinned_piece_position,
                        stm_pawn_pin_enpassant.trailing_zeros() as usize,
                        PieceType::Pawn,
                        GameMoveType::EnPassant,
                    );
                }
            }
            enemy_bishop_on_xray ^= enemy_bishop;
        }
    }

    //----------------------------------------------------------------------
    //**********************************************************************
    //5. Pawn pushes, captures, and promotions (captures, capture-enpassant, capture-promotion, normal-promotion)
    //5.1 Single push (promotions and pushes)
    let stm_pawns_single_push = if stm_color_iswhite {
        w_single_push_pawn_targets(side_pawns, empty_squares)
    } else {
        b_single_push_pawn_targets(side_pawns, empty_squares)
    } & push_mask;
    let stm_pawn_promotions = stm_pawns_single_push & RANKS[if stm_color_iswhite { 7 } else { 0 }];
    if !only_captures {
        add_pawn_moves_to_movelist(
            g,
            movelist,
            stm_pawn_promotions,
            8,
            false,
            true,
            pinned_pieces,
        );
    }
    if !only_captures {
        let stm_pawns_quiet_single_push = stm_pawns_single_push & !stm_pawn_promotions;
        add_pawn_moves_to_movelist(
            g,
            movelist,
            stm_pawns_quiet_single_push,
            8,
            false,
            false,
            pinned_pieces,
        );
    }
    //5.2 Double push
    if !only_captures {
        let stm_pawns_double_push = if stm_color_iswhite {
            w_double_push_pawn_targets(side_pawns, empty_squares)
        } else {
            b_double_push_pawn_targets(side_pawns, empty_squares)
        } & push_mask;
        add_pawn_moves_to_movelist(
            g,
            movelist,
            stm_pawns_double_push,
            16,
            false,
            false,
            pinned_pieces,
        );
    }
    //5.3 West captures (normal capture, promotion capture, en passant)
    let west_targets = pawn_west_targets(side, side_pawns);
    let stm_pawn_west_captures = west_targets & capture_mask & enemy_pieces;
    //Split up in promotion and non-promotion captures
    let stm_pawn_west_promotion_capture =
        stm_pawn_west_captures & RANKS[if stm_color_iswhite { 7 } else { 0 }];
    add_pawn_moves_to_movelist(
        g,
        movelist,
        stm_pawn_west_promotion_capture,
        7,
        true,
        true,
        pinned_pieces,
    );
    let stm_pawn_west_nonpromotion_capture =
        stm_pawn_west_captures & !stm_pawn_west_promotion_capture;
    add_pawn_moves_to_movelist(
        g,
        movelist,
        stm_pawn_west_nonpromotion_capture,
        7,
        true,
        false,
        pinned_pieces,
    );
    //En passants
    let stm_pawn_west_enpassants = west_targets
        & g.get_en_passant()
        & if stm_color_iswhite {
            capture_mask << 8
        } else {
            capture_mask >> 8
        };
    if stm_pawn_west_enpassants != 0u64
        && if stm_color_iswhite {
            stm_pawn_west_enpassants >> 7
        } else {
            stm_pawn_west_enpassants << 7
        } & pinned_pieces
            == 0u64
    {
        let pawn_index = stm_pawn_west_enpassants.trailing_zeros() as usize;
        let (pawn_from, removed_piece_index) = if stm_color_iswhite {
            (pawn_index - 7, pawn_index - 8)
        } else {
            (pawn_index + 7, pawn_index + 8)
        };
        let all_pieces_without_en_passants =
            all_pieces & !square(pawn_from) & !square(removed_piece_index);
        if rook_attack(g.get_king_square(side), all_pieces_without_en_passants)
            & RANKS[g.get_king_square(side) / 8]
            & g.get_rook_like_bb(enemy)
            == 0u64
        {
            add_move_to_movelist(
                movelist,
                pawn_from,
                pawn_index,
                PieceType::Pawn,
                GameMoveType::EnPassant,
            );
        }
    }
    //5.4 East captures (normal capture, promotion capture, en passant)
    let east_targets = pawn_east_targets(side, side_pawns);
    let stm_pawn_east_captures = east_targets & capture_mask & enemy_pieces;
    //Split up in promotion and non-promotion captures
    let stm_pawn_east_promotion_capture =
        stm_pawn_east_captures & RANKS[if stm_color_iswhite { 7 } else { 0 }];
    add_pawn_moves_to_movelist(
        g,
        movelist,
        stm_pawn_east_promotion_capture,
        9,
        true,
        true,
        pinned_pieces,
    );
    let stm_pawn_east_nonpromotion_capture =
        stm_pawn_east_captures & !stm_pawn_east_promotion_capture;
    add_pawn_moves_to_movelist(
        g,
        movelist,
        stm_pawn_east_nonpromotion_capture,
        9,
        true,
        false,
        pinned_pieces,
    );
    //En passants
    let stm_pawn_east_enpassants = east_targets
        & g.get_en_passant()
        & if stm_color_iswhite {
            capture_mask << 8
        } else {
            capture_mask >> 8
        };
    if stm_pawn_east_enpassants != 0u64
        && if stm_color_iswhite {
            stm_pawn_east_enpassants >> 9
        } else {
            stm_pawn_east_enpassants << 9
        } & pinned_pieces
            == 0u64
    {
        let pawn_index = stm_pawn_east_enpassants.trailing_zeros() as usize;
        let (pawn_from, removed_piece_index) = if stm_color_iswhite {
            (pawn_index - 9, pawn_index - 8)
        } else {
            (pawn_index + 9, pawn_index + 8)
        };
        let all_pieces_without_en_passants =
            all_pieces & !square(pawn_from) & !square(removed_piece_index);
        if rook_attack(g.get_king_square(side), all_pieces_without_en_passants)
            & RANKS[g.get_king_square(side) / 8]
            & g.get_rook_like_bb(enemy)
            == 0u64
        {
            add_move_to_movelist(
                movelist,
                pawn_from,
                pawn_index,
                PieceType::Pawn,
                GameMoveType::EnPassant,
            );
        }
    }

    //----------------------------------------------------------------------
    //**********************************************************************
    //6. All other legal moves (knights, bishops, rooks, queens)
    for pt in [
        PieceType::Knight,
        PieceType::Queen,
        PieceType::Bishop,
        PieceType::Rook,
    ]
    .iter()
    {
        add_normal_moves_to_movelist(
            g,
            movelist,
            *pt,
            g.get_piece(*pt, side),
            pinned_pieces,
            enemy_pieces,
            empty_squares,
            push_mask,
            capture_mask,
            only_captures,
        )
    }
    //----------------------------------------------------------------------
    //**********************************************************************
    //7. Castling
    if !only_captures && checkers == 0 {
        if stm_color_iswhite {
            if g.castle_white_kingside()
                && (all_pieces | enemy_attacks) & (square(square::F1) | square(square::G1)) == 0u64
            {
                movelist.add_move(GameMove {
                    from: g.get_king_square(side) as u8,
                    to: square::G1 as u8,
                    move_type: GameMoveType::Castle,
                    piece_type: PieceType::King,
                });
            }
            if g.castle_white_queenside()
                && ((all_pieces | enemy_attacks) & (square(square::C1) | square(square::D1))
                    | all_pieces & square(square::B1))
                    == 0u64
            {
                movelist.add_move(GameMove {
                    from: g.get_king_square(side) as u8,
                    to: square::C1 as u8,
                    move_type: GameMoveType::Castle,
                    piece_type: PieceType::King,
                });
            }
        } else {
            if g.castle_black_kingside()
                && (all_pieces | enemy_attacks) & (square(square::F8) | square(square::G8)) == 0u64
            {
                movelist.add_move(GameMove {
                    from: g.get_king_square(side) as u8,
                    to: square::G8 as u8,
                    move_type: GameMoveType::Castle,
                    piece_type: PieceType::King,
                });
            }
            if g.castle_black_queenside()
                && ((all_pieces | enemy_attacks) & (square(square::C8) | square(square::D8))
                    | all_pieces & square(square::B8))
                    == 0u64
            {
                movelist.add_move(GameMove {
                    from: g.get_king_square(side) as u8,
                    to: square::C8 as u8,
                    move_type: GameMoveType::Castle,
                    piece_type: PieceType::King,
                });
            }
        }
    }
    //----------------------------------------------------------------------
    AdditionalGameStateInformation { stm_incheck }
}
