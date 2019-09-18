use super::super::bitboards;
use super::super::board_representation::game_state::{
    GameMove, GameMoveType, GameState, PieceType, BISHOP, KING, KNIGHT, PAWN, QUEEN, ROOK, WHITE,
};
use super::magic::{self, Magic};
use crate::board_representation::game_state_attack_container::{
    GameStateAttackContainer, MGSA_BISHOP, MGSA_KNIGHT, MGSA_QUEEN, MGSA_ROOKS,
};
use crate::search::GradedMove;

//Movegen
//King - piecewise by lookup
//Knight - piecewise by lookup
//Bishop/Queen/Rook - piecewise by lookup in magic
//Pawn - setwise by shift
#[inline(always)]
pub fn king_attack(square: usize) -> u64 {
    bitboards::KING_ATTACKS[square]
}
#[inline(always)]
pub fn bishop_attack(square: usize, all_pieces: u64) -> u64 {
    Magic::get_attacks(&magic::MAGICS_BISHOPS[square], all_pieces)
}
#[inline(always)]
pub fn rook_attack(square: usize, all_pieces: u64) -> u64 {
    Magic::get_attacks(&magic::MAGICS_ROOKS[square], all_pieces)
}
#[inline(always)]
pub fn knight_attack(square: usize) -> u64 {
    bitboards::KNIGHT_ATTACKS[square]
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
    bitboards::north_one(bitboards::north_one(pawns & bitboards::RANKS[1]) & empty) & empty
}

#[inline(always)]
pub fn b_double_push_pawn_targets(pawns: u64, empty: u64) -> u64 {
    bitboards::south_one(bitboards::south_one(pawns & bitboards::RANKS[6]) & empty) & empty
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
    let to_board = 1u64 << to;
    if g.pieces[PAWN][1 - g.color_to_move] & to_board != 0u64 {
        PieceType::Pawn
    } else if g.pieces[KNIGHT][1 - g.color_to_move] & to_board != 0u64 {
        PieceType::Knight
    } else if g.pieces[QUEEN][1 - g.color_to_move] & to_board != 0u64 {
        PieceType::Queen
    } else if g.pieces[BISHOP][1 - g.color_to_move] & to_board != 0u64 {
        PieceType::Bishop
    } else if g.pieces[ROOK][1 - g.color_to_move] & to_board != 0u64 {
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
pub fn get_rook_ray(king_square: usize, rook_square: usize) -> u64 {
    bitboards::ROOK_RAYS[king_square][rook_square]
}

#[inline(always)]
pub fn get_bishop_ray(king_square: usize, bishop_square: usize) -> u64 {
    bitboards::BISHOP_RAYS[king_square][bishop_square]
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
) -> bool {
    let pin_quiet_targets = ray_to_king & push_mask & !(1u64 << pinned_piece_position);
    let pin_capture_possible = (capture_mask & enemy_pinner) != 0u64;
    let haslegalmove = pin_capture_possible || pin_quiet_targets != 0u64;
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
    haslegalmove
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
        captures ^= 1u64 << capture_index;
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
) -> bool {
    let mut stm_haslegalmove = false;
    while target_board != 0u64 {
        let pawn_index = target_board.trailing_zeros() as usize;
        let pawn = 1u64 << pawn_index;
        let from_index = if g.color_to_move == WHITE {
            pawn_index - shift
        } else {
            pawn_index + shift
        };
        let from_board = 1u64 << from_index;
        if from_board & pinned_pieces == 0u64 {
            stm_haslegalmove = true;
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
    stm_haslegalmove
}

#[inline(always)]
pub fn add_normal_moves_to_movelist(
    g: &GameState,
    legal_moves: &mut MoveList,
    attack_container: &GameStateAttackContainer,
    piece_type: PieceType,
    mut piece_board: u64,
    pinned_pieces: u64,
    enemy_pieces: u64,
    empty_squares: u64,
    push_mask: u64,
    capture_mask: u64,
    only_captures: bool,
) -> bool {
    let mut stm_haslegalmove = false;
    let mut index = 0;
    while piece_board != 0u64 {
        let piece_index = piece_board.trailing_zeros() as usize;
        let piece = 1u64 << piece_index;
        if piece & pinned_pieces == 0u64 {
            let piece_target = if let PieceType::Knight = piece_type {
                attack_container.attack[MGSA_KNIGHT][g.color_to_move][index]
            } else if let PieceType::Bishop = piece_type {
                attack_container.attack[MGSA_BISHOP][g.color_to_move][index]
            } else if let PieceType::Rook = piece_type {
                attack_container.attack[MGSA_ROOKS][g.color_to_move][index]
            } else if let PieceType::Queen = piece_type {
                attack_container.attack[MGSA_QUEEN][g.color_to_move][index]
            } else {
                panic!("Shouldn't get here")
            };
            let mut captures = piece_target & capture_mask & enemy_pieces;
            stm_haslegalmove |= captures != 0u64;
            while captures != 0u64 {
                let capture_index = captures.trailing_zeros() as usize;
                add_move_to_movelist(
                    legal_moves,
                    piece_index,
                    capture_index,
                    piece_type,
                    GameMoveType::Capture(find_captured_piece_type(g, capture_index)),
                );
                captures ^= 1u64 << capture_index;
            }

            if !only_captures || !stm_haslegalmove {
                let quiets = piece_target & push_mask & empty_squares;
                stm_haslegalmove |= quiets != 0u64;
                if !only_captures {
                    add_moves_to_movelist(
                        legal_moves,
                        piece_index,
                        quiets,
                        piece_type,
                        GameMoveType::Quiet,
                    );
                }
            }
        }
        piece_board ^= piece;
        index += 1;
    }
    stm_haslegalmove
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
        target_board ^= 1u64 << target_square;
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
        from: from_square,
        to: to_square,
        move_type,
        piece_type,
    });
}

#[inline(always)]
pub fn get_checkers(game_state: &GameState, early_exit: bool) -> u64 {
    let mut checkers = 0u64;
    let my_king = game_state.pieces[KING][game_state.color_to_move];
    checkers |= knight_attack(my_king.trailing_zeros() as usize)
        & game_state.pieces[KNIGHT][1 - game_state.color_to_move];
    checkers |= (pawn_west_targets(game_state.color_to_move, my_king)
        | pawn_east_targets(game_state.color_to_move, my_king))
        & game_state.pieces[PAWN][1 - game_state.color_to_move];
    if early_exit && checkers != 0u64 {
        return checkers;
    }
    let all_pieces = game_state.get_all_pieces();
    checkers |= bishop_attack(my_king.trailing_zeros() as usize, all_pieces)
        & (game_state.pieces[BISHOP][1 - game_state.color_to_move]
            | game_state.pieces[QUEEN][1 - game_state.color_to_move]);
    if early_exit && checkers != 0u64 {
        return checkers;
    }
    checkers |= rook_attack(my_king.trailing_zeros() as usize, all_pieces)
        & (game_state.pieces[ROOK][1 - game_state.color_to_move]
            | game_state.pieces[QUEEN][1 - game_state.color_to_move]);
    checkers
}
#[derive(Clone)]
pub struct AdditionalGameStateInformation {
    pub stm_incheck: bool,
    pub stm_haslegalmove: bool,
}
pub const MAX_MOVES: usize = 128;
pub struct MoveList {
    pub move_list: Vec<Option<GameMove>>,
    pub graded_moves: Vec<Option<GradedMove>>,
    pub counter: usize,
}

impl Default for MoveList {
    fn default() -> Self {
        let mut move_list = Vec::with_capacity(MAX_MOVES);
        let mut graded_moves = Vec::with_capacity(MAX_MOVES);
        for _ in 0..MAX_MOVES {
            move_list.push(None);
            graded_moves.push(None);
        }
        MoveList {
            move_list,
            graded_moves,
            counter: 0,
        }
    }
}
impl MoveList {
    pub fn add_move(&mut self, mv: GameMove) {
        self.move_list[self.counter] = Some(mv);
        self.counter += 1;
    }
}

pub fn generate_moves(
    g: &GameState,
    only_captures: bool,
    movelist: &mut MoveList,
    attack_container: &GameStateAttackContainer,
) -> AdditionalGameStateInformation {
    //----------------------------------------------------------------------
    //**********************************************************************
    //1. General bitboards and variable initialization
    movelist.counter = 0;

    let side = g.color_to_move;
    let enemy = 1 - side;
    let stm_color_iswhite: bool = g.color_to_move == WHITE;

    let mut side_pawns = g.pieces[PAWN][side];
    let side_pieces = g.get_pieces_from_side(side);
    let enemy_pieces = g.get_pieces_from_side(enemy);
    let all_pieces = enemy_pieces | side_pieces;
    let empty_squares = !all_pieces;

    let mut stm_haslegalmove = false;

    //----------------------------------------------------------------------
    //**********************************************************************
    //2. Safe King moves
    let stm_legal_kingmoves =
        attack_container.king_attacks[side] & !attack_container.attacks_sum[enemy] & !side_pieces;
    stm_haslegalmove |= stm_legal_kingmoves != 0u64;
    add_king_moves_to_movelist(
        g,
        movelist,
        only_captures,
        stm_legal_kingmoves,
        g.king_square(side),
        enemy_pieces,
    );
    //----------------------------------------------------------------------
    //**********************************************************************
    //3. Check & Check Evasions
    let check_board = get_checkers(&g, false);
    let checkers = check_board.count_ones() as usize;
    let stm_incheck = checkers > 0;

    let mut capture_mask = 0xFFFF_FFFF_FFFF_FFFFu64;
    let mut push_mask = 0xFFFF_FFFF_FFFF_FFFFu64;
    if checkers > 1 {
        //Double check, only safe king moves are legal
        return AdditionalGameStateInformation {
            stm_incheck,
            stm_haslegalmove,
        };
    } else if checkers == 1 {
        //Only a single checker
        capture_mask = check_board;
        //If it's a slider, we can also push in its way
        if check_board & (g.pieces[BISHOP][enemy] | g.pieces[ROOK][enemy] | g.pieces[QUEEN][enemy])
            != 0u64
        {
            let checker_square = check_board.trailing_zeros() as usize;
            if check_board & (bitboards::FREEFIELD_ROOK_ATTACKS[g.king_square(side)]) != 0u64 {
                //Checker is rook-like
                push_mask = get_rook_ray(g.king_square(side), checker_square);
            } else {
                //Checker is bishop-like
                push_mask = get_bishop_ray(g.king_square(side), checker_square);
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
    let stm_rook_attacks_from_king = rook_attack(g.king_square(side), all_pieces);
    let stm_xray_rook_attacks_from_king = xray_rook_attacks(
        stm_rook_attacks_from_king,
        all_pieces,
        side_pieces,
        g.king_square(side),
    );
    let mut enemy_rooks_on_xray =
        stm_xray_rook_attacks_from_king & (g.pieces[ROOK][enemy] | g.pieces[QUEEN][enemy]);
    while enemy_rooks_on_xray != 0u64 {
        let enemy_rook_position = enemy_rooks_on_xray.trailing_zeros() as usize;
        let enemy_rook = 1u64 << enemy_rook_position;
        let ray_to_king = get_rook_ray(g.king_square(side), enemy_rook_position);
        let pinned_piece = ray_to_king & side_pieces;
        let pinned_piece_position = pinned_piece.trailing_zeros() as usize;
        pinned_pieces |= pinned_piece;
        if pinned_piece & g.pieces[QUEEN][side] != 0u64 {
            //Add possible queen pushes
            stm_haslegalmove |= add_pin_moves_to_movelist(
                movelist,
                only_captures,
                ray_to_king,
                push_mask,
                capture_mask,
                enemy_rook,
                pinned_piece_position,
                PieceType::Queen,
                enemy_rook_position,
                g.pieces[QUEEN][enemy],
                PieceType::Rook,
            );
        } else if pinned_piece & g.pieces[ROOK][side] != 0u64 {
            //Add possible rook pushes
            stm_haslegalmove |= add_pin_moves_to_movelist(
                movelist,
                only_captures,
                ray_to_king,
                push_mask,
                capture_mask,
                enemy_rook,
                pinned_piece_position,
                PieceType::Rook,
                enemy_rook_position,
                g.pieces[QUEEN][enemy],
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
            stm_haslegalmove |= (stm_pawn_pin_single_push | stm_pawn_pin_double_push) != 0u64;
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
    //4.2 Bishop-Like pins
    let stm_bishop_attacks_from_king = bishop_attack(g.king_square(side), all_pieces);
    let stm_xray_bishop_attacks_from_king = xray_bishop_attacks(
        stm_bishop_attacks_from_king,
        all_pieces,
        side_pieces,
        g.king_square(side),
    );
    let mut enemy_bishop_on_xray =
        stm_xray_bishop_attacks_from_king & (g.pieces[BISHOP][enemy] | g.pieces[QUEEN][enemy]);
    while enemy_bishop_on_xray != 0u64 {
        let enemy_bishop_position = enemy_bishop_on_xray.trailing_zeros() as usize;
        let enemy_bishop = 1u64 << enemy_bishop_position;
        let ray_to_king = get_bishop_ray(g.king_square(side), enemy_bishop_position);
        let pinned_piece = ray_to_king & side_pieces;
        let pinned_piece_position = pinned_piece.trailing_zeros() as usize;
        pinned_pieces |= pinned_piece;
        if pinned_piece & g.pieces[QUEEN][side] != 0u64 {
            //Add possible queen pushes
            stm_haslegalmove |= add_pin_moves_to_movelist(
                movelist,
                only_captures,
                ray_to_king,
                push_mask,
                capture_mask,
                enemy_bishop,
                pinned_piece_position,
                PieceType::Queen,
                enemy_bishop_position,
                g.pieces[QUEEN][enemy],
                PieceType::Bishop,
            );
        } else if pinned_piece & g.pieces[BISHOP][side] != 0u64 {
            //Add possible bishop pushes
            stm_haslegalmove |= add_pin_moves_to_movelist(
                movelist,
                only_captures,
                ray_to_king,
                push_mask,
                capture_mask,
                enemy_bishop,
                pinned_piece_position,
                PieceType::Bishop,
                enemy_bishop_position,
                g.pieces[QUEEN][enemy],
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
                stm_pawn_pin_captures & bitboards::RANKS[if stm_color_iswhite { 7 } else { 0 }];
            if stm_pawn_pin_promotion_capture != 0u64 {
                stm_haslegalmove = true;
                add_promotion_move_to_movelist(
                    movelist,
                    pinned_piece_position,
                    enemy_bishop_position,
                    GameMoveType::Capture(if enemy_bishop & g.pieces[QUEEN][enemy] != 0u64 {
                        PieceType::Queen
                    } else {
                        PieceType::Bishop
                    }),
                );
            }
            let stm_pawn_pin_nonpromotion_capture =
                stm_pawn_pin_captures & !stm_pawn_pin_promotion_capture;
            if stm_pawn_pin_nonpromotion_capture != 0u64 {
                stm_haslegalmove = true;
                add_move_to_movelist(
                    movelist,
                    pinned_piece_position,
                    enemy_bishop_position,
                    PieceType::Pawn,
                    GameMoveType::Capture(if enemy_bishop & g.pieces[QUEEN][enemy] != 0u64 {
                        PieceType::Queen
                    } else {
                        PieceType::Bishop
                    }),
                );
            }
            //En passants
            let stm_pawn_pin_enpassant =
                stm_pawn_pin_target & g.en_passant & capture_mask & ray_to_king;
            if stm_pawn_pin_enpassant != 0u64 {
                stm_haslegalmove = true;
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

    //----------------------------------------------------------------------
    //**********************************************************************
    //5. Pawn pushes, captures, and promotions (captures, capture-enpassant, capture-promotion, normal-promotion)
    //5.1 Single push (promotions and pushes)
    let stm_pawns_single_push = if stm_color_iswhite {
        w_single_push_pawn_targets(side_pawns, empty_squares)
    } else {
        b_single_push_pawn_targets(side_pawns, empty_squares)
    } & push_mask;
    stm_haslegalmove |= stm_pawns_single_push != 0u64;
    let stm_pawn_promotions =
        stm_pawns_single_push & bitboards::RANKS[if stm_color_iswhite { 7 } else { 0 }];
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
    if !only_captures || !stm_haslegalmove {
        let stm_pawns_double_push = if stm_color_iswhite {
            w_double_push_pawn_targets(side_pawns, empty_squares)
        } else {
            b_double_push_pawn_targets(side_pawns, empty_squares)
        } & push_mask;
        stm_haslegalmove |= stm_pawns_double_push != 0u64;
        if !only_captures {
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
    }
    //5.3 West captures (normal capture, promotion capture, en passant)
    let stm_pawn_west_captures =
        attack_container.pawn_west_attacks[side] & capture_mask & enemy_pieces;
    //Split up in promotion and non-promotion captures
    let stm_pawn_west_promotion_capture =
        stm_pawn_west_captures & bitboards::RANKS[if stm_color_iswhite { 7 } else { 0 }];
    stm_haslegalmove |= add_pawn_moves_to_movelist(
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
    stm_haslegalmove |= add_pawn_moves_to_movelist(
        g,
        movelist,
        stm_pawn_west_nonpromotion_capture,
        7,
        true,
        false,
        pinned_pieces,
    );
    //En passants
    let stm_pawn_west_enpassants = attack_container.pawn_west_attacks[side]
        & g.en_passant
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
            all_pieces & !(1u64 << pawn_from) & !(1u64 << removed_piece_index);
        if rook_attack(g.king_square(side), all_pieces_without_en_passants)
            & bitboards::RANKS[g.king_square(side) / 8]
            & (g.pieces[ROOK][enemy] | g.pieces[QUEEN][enemy])
            == 0u64
        {
            stm_haslegalmove = true;
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
    let stm_pawn_east_captures =
        attack_container.pawn_east_attacks[side] & capture_mask & enemy_pieces;
    //Split up in promotion and non-promotion captures
    let stm_pawn_east_promotion_capture =
        stm_pawn_east_captures & bitboards::RANKS[if stm_color_iswhite { 7 } else { 0 }];
    stm_haslegalmove |= add_pawn_moves_to_movelist(
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
    stm_haslegalmove |= add_pawn_moves_to_movelist(
        g,
        movelist,
        stm_pawn_east_nonpromotion_capture,
        9,
        true,
        false,
        pinned_pieces,
    );
    //En passants
    let stm_pawn_east_enpassants = attack_container.pawn_east_attacks[side]
        & g.en_passant
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
            all_pieces & !(1u64 << pawn_from) & !(1u64 << removed_piece_index);
        if rook_attack(g.king_square(side), all_pieces_without_en_passants)
            & bitboards::RANKS[g.king_square(side) / 8]
            & (g.pieces[ROOK][enemy] | g.pieces[QUEEN][enemy])
            == 0u64
        {
            stm_haslegalmove = true;
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
    //6.1 Knights
    stm_haslegalmove |= add_normal_moves_to_movelist(
        g,
        movelist,
        attack_container,
        PieceType::Knight,
        g.pieces[KNIGHT][side],
        pinned_pieces,
        enemy_pieces,
        empty_squares,
        push_mask,
        capture_mask,
        only_captures,
    );
    //6.4 Queens
    stm_haslegalmove |= add_normal_moves_to_movelist(
        g,
        movelist,
        attack_container,
        PieceType::Queen,
        g.pieces[QUEEN][side],
        pinned_pieces,
        enemy_pieces,
        empty_squares,
        push_mask,
        capture_mask,
        only_captures,
    );

    //6.2 Bishops
    stm_haslegalmove |= add_normal_moves_to_movelist(
        g,
        movelist,
        attack_container,
        PieceType::Bishop,
        g.pieces[BISHOP][side],
        pinned_pieces,
        enemy_pieces,
        empty_squares,
        push_mask,
        capture_mask,
        only_captures,
    );
    //6.3 Rooks
    stm_haslegalmove |= add_normal_moves_to_movelist(
        g,
        movelist,
        attack_container,
        PieceType::Rook,
        g.pieces[ROOK][side],
        pinned_pieces,
        enemy_pieces,
        empty_squares,
        push_mask,
        capture_mask,
        only_captures,
    );
    //----------------------------------------------------------------------
    //**********************************************************************
    //7. Castling
    if (!only_captures || !stm_haslegalmove) && checkers == 0 {
        if stm_color_iswhite {
            if g.castle_white_kingside
                && (all_pieces | attack_container.attacks_sum[enemy])
                    & (bitboards::SQUARES[5] | bitboards::SQUARES[6])
                    == 0u64
            {
                stm_haslegalmove = true;
                if !only_captures {
                    movelist.add_move(GameMove {
                        from: g.king_square(side),
                        to: 6usize,
                        move_type: GameMoveType::Castle,
                        piece_type: PieceType::King,
                    });
                }
            }
            if g.castle_white_queenside
                && ((all_pieces | attack_container.attacks_sum[enemy])
                    & (bitboards::SQUARES[2] | bitboards::SQUARES[3])
                    | all_pieces & bitboards::SQUARES[1])
                    == 0u64
            {
                stm_haslegalmove = true;
                if !only_captures {
                    movelist.add_move(GameMove {
                        from: g.king_square(side),
                        to: 2usize,
                        move_type: GameMoveType::Castle,
                        piece_type: PieceType::King,
                    });
                }
            }
        } else {
            if g.castle_black_kingside
                && (all_pieces | attack_container.attacks_sum[enemy])
                    & (bitboards::SQUARES[61] | bitboards::SQUARES[62])
                    == 0u64
            {
                stm_haslegalmove = true;
                if !only_captures {
                    movelist.add_move(GameMove {
                        from: g.king_square(side),
                        to: 62usize,
                        move_type: GameMoveType::Castle,
                        piece_type: PieceType::King,
                    });
                }
            }
            if g.castle_black_queenside
                && ((all_pieces | attack_container.attacks_sum[enemy])
                    & (bitboards::SQUARES[58] | bitboards::SQUARES[59])
                    | all_pieces & bitboards::SQUARES[57])
                    == 0u64
            {
                stm_haslegalmove = true;
                if !only_captures {
                    movelist.add_move(GameMove {
                        from: g.king_square(side),
                        to: 58usize,
                        move_type: GameMoveType::Castle,
                        piece_type: PieceType::King,
                    });
                }
            }
        }
    }
    //----------------------------------------------------------------------
    AdditionalGameStateInformation {
        stm_incheck,
        stm_haslegalmove,
    }
}
