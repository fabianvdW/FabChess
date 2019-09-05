use super::super::bitboards;
use super::super::board_representation::game_state::{
    self, GameMove, GameMoveType, PieceType, BISHOP, BLACK, KING, KNIGHT, PAWN, QUEEN, ROOK, WHITE,
};
use super::magic::{self, Magic};
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
pub fn find_captured_piece_type(
    to: usize,
    e_pawns: u64,
    e_knights: u64,
    e_bishops: u64,
    e_rooks: u64,
    e_queens: u64,
) -> PieceType {
    let to_board = 1u64 << to;
    if e_pawns & to_board != 0u64 {
        PieceType::Pawn
    } else if e_knights & to_board != 0u64 {
        PieceType::Knight
    //Find queens before bishops and rooks since it's in all three boards
    } else if e_queens & to_board != 0u64 {
        PieceType::Queen
    } else if e_bishops & to_board != 0u64 {
        PieceType::Bishop
    } else if e_rooks & to_board != 0u64 {
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

#[derive(Clone)]
pub struct AdditionalBitBoards {
    stm_pieces_without_king: u64,
    stm_pieces: u64,
    enemy_pieces: u64,
    all_pieces_without_stmking: u64,
    all_pieces: u64,

    stm_pawns_westattack: u64,
    stm_pawns_eastattack: u64,
    enemy_pawns_westattack: u64,
    enemy_pawns_eastattack: u64,

    stm_king_attacks: u64,
    enemy_king_attacks: u64,

    stm_unsafe_squares: u64,
    all_checkers: u64,
}

pub fn calculate_additionalbitboards(
    stm_pawns: u64,
    enemy_pawns: u64,
    stm_knights: u64,
    mut enemy_knights: u64,
    stm_bishops: u64,
    mut enemy_bishops: u64,
    stm_rooks: u64,
    mut enemy_rooks: u64,
    stm_queens: u64,
    mut enemy_queens: u64,
    stm_king: u64,
    enemy_king: u64,
    stm_color_iswhite: bool,
) -> AdditionalBitBoards {
    let stm_pieces_without_king: u64 =
        stm_pawns | stm_knights | stm_bishops | stm_rooks | stm_queens;
    let stm_pieces: u64 = stm_pieces_without_king | stm_king;
    let enemy_pieces =
        enemy_pawns | enemy_knights | enemy_bishops | enemy_rooks | enemy_queens | enemy_king;
    let all_pieces_without_stmking = enemy_pieces | stm_pieces_without_king;
    let all_pieces = all_pieces_without_stmking | stm_king;

    let (mut stm_unsafe_squares, mut all_checkers) = (0u64, 0u64);

    //Pawns
    let side = if stm_color_iswhite { WHITE } else { BLACK };
    let (
        stm_pawns_westattack,
        stm_pawns_eastattack,
        enemy_pawns_westattack,
        enemy_pawns_eastattack,
    ) = (
        pawn_west_targets(side, stm_pawns),
        pawn_east_targets(side, stm_pawns),
        pawn_west_targets(1 - side, enemy_pawns),
        pawn_east_targets(1 - side, enemy_pawns),
    );

    stm_unsafe_squares |= enemy_pawns_westattack | enemy_pawns_eastattack;

    all_checkers |= pawn_west_targets(side, stm_king & enemy_pawns_westattack)
        | pawn_east_targets(side, stm_king & enemy_pawns_eastattack);

    //Knights
    while enemy_knights != 0u64 {
        let enemy_knightindex = enemy_knights.trailing_zeros() as usize;
        let enemy_knight = 1u64 << enemy_knightindex;
        let enemy_knightattacks = knight_attack(enemy_knightindex);
        stm_unsafe_squares |= enemy_knightattacks;
        if stm_king & enemy_knightattacks != 0u64 {
            all_checkers |= enemy_knight;
        }
        enemy_knights ^= enemy_knight;
    }

    //Bishops
    while enemy_bishops != 0u64 {
        let enemy_bishopindex = enemy_bishops.trailing_zeros() as usize;
        let enemy_bishop = 1u64 << enemy_bishopindex;
        let enemy_bishopattacks = bishop_attack(enemy_bishopindex, all_pieces_without_stmking);
        stm_unsafe_squares |= enemy_bishopattacks;
        if stm_king & enemy_bishopattacks != 0u64 {
            all_checkers |= enemy_bishop;
        }
        enemy_bishops ^= enemy_bishop;
    }

    //Rooks
    while enemy_rooks != 0u64 {
        let enemy_rookindex = enemy_rooks.trailing_zeros() as usize;
        let enemy_rook = 1u64 << enemy_rookindex;
        let enemy_rookattacks = rook_attack(enemy_rookindex, all_pieces_without_stmking);
        stm_unsafe_squares |= enemy_rookattacks;
        if stm_king & enemy_rookattacks != 0u64 {
            all_checkers |= enemy_rook;
        }
        enemy_rooks ^= enemy_rook;
    }

    //Queens
    while enemy_queens != 0u64 {
        let enemy_queenindex = enemy_queens.trailing_zeros() as usize;
        let enemy_queen = 1u64 << enemy_queenindex;
        let enemy_queenattacks = bishop_attack(enemy_queenindex, all_pieces_without_stmking)
            | rook_attack(enemy_queenindex, all_pieces_without_stmking);
        stm_unsafe_squares |= enemy_queenattacks;
        if stm_king & enemy_queenattacks != 0u64 {
            all_checkers |= enemy_queen;
        }
        enemy_queens ^= enemy_queen;
    }
    //Kings
    let stm_king_attacks = king_attack(stm_king.trailing_zeros() as usize) & !stm_pieces;
    let enemy_king_attacks = king_attack(enemy_king.trailing_zeros() as usize);
    stm_unsafe_squares |= enemy_king_attacks;
    AdditionalBitBoards {
        stm_pieces_without_king,
        stm_pieces,
        enemy_pieces,
        all_pieces_without_stmking,
        all_pieces,
        stm_pawns_westattack,
        stm_pawns_eastattack,
        enemy_pawns_westattack,
        enemy_pawns_eastattack,
        stm_king_attacks,
        enemy_king_attacks,
        stm_unsafe_squares,
        all_checkers,
    }
}

#[derive(Clone)]
pub struct AdditionalGameStateInformation {
    pub stm_incheck: bool,
    pub stm_haslegalmove: bool,
    pub additional_bitboards: AdditionalBitBoards,
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
    depth: usize,
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
            depth,
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
            depth,
        );
    }
    haslegalmove
}
#[inline(always)]
pub fn add_king_moves_to_movelist(
    legal_moves: &mut MoveList,
    only_captures: bool,
    stm_legal_kingmoves: u64,
    stm_king_index: usize,
    enemy_pawns: u64,
    enemy_knights: u64,
    enemy_bishops: u64,
    enemy_rooks: u64,
    enemy_queens: u64,
    enemy_pieces: u64,
    depth: usize,
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
            GameMoveType::Capture(find_captured_piece_type(
                capture_index,
                enemy_pawns,
                enemy_knights,
                enemy_bishops,
                enemy_rooks,
                enemy_queens,
            )),
            depth,
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
            depth,
        );
    }
}

#[inline(always)]
pub fn add_pawn_moves_to_movelist(
    legal_moves: &mut MoveList,
    mut target_board: u64,
    shift: usize,
    stm_color_iswhite: bool,
    enemy_pawns: u64,
    enemy_knights: u64,
    enemy_bishops: u64,
    enemy_rooks: u64,
    enemy_queens: u64,
    is_capture: bool,
    is_promotion: bool,
    pinned_pieces: u64,
    depth: usize,
) -> bool {
    let mut stm_haslegalmove = false;
    while target_board != 0u64 {
        let pawn_index = target_board.trailing_zeros() as usize;
        let pawn = 1u64 << pawn_index;
        let from_index = if stm_color_iswhite {
            pawn_index - shift
        } else {
            pawn_index + shift
        };
        let from_board = 1u64 << from_index;
        if from_board & pinned_pieces == 0u64 {
            stm_haslegalmove = true;
            let mv_type = if is_capture {
                GameMoveType::Capture(find_captured_piece_type(
                    pawn_index,
                    enemy_pawns,
                    enemy_knights,
                    enemy_bishops,
                    enemy_rooks,
                    enemy_queens,
                ))
            } else {
                GameMoveType::Quiet
            };
            if is_promotion {
                add_promotion_move_to_movelist(legal_moves, from_index, pawn_index, mv_type, depth);
            } else {
                add_move_to_movelist(
                    legal_moves,
                    from_index,
                    pawn_index,
                    PieceType::Pawn,
                    mv_type,
                    depth,
                )
            }
        }
        target_board ^= pawn;
    }
    stm_haslegalmove
}
#[inline(always)]
pub fn add_normal_moves_to_movelist(
    legal_moves: &mut MoveList,
    piece_type: PieceType,
    mut piece_board: u64,
    pinned_pieces: u64,
    enemy_pawns: u64,
    enemy_knights: u64,
    enemy_bishops: u64,
    enemy_rooks: u64,
    enemy_queens: u64,
    enemy_pieces: u64,
    empty_squares: u64,
    all_pieces: u64,
    push_mask: u64,
    capture_mask: u64,
    only_captures: bool,
    depth: usize,
    stm_color_iswhite: bool,
) -> bool {
    let mut stm_haslegalmove = false;
    while piece_board != 0u64 {
        let piece_index = if stm_color_iswhite {
            63 - piece_board.leading_zeros() as usize
        } else {
            piece_board.trailing_zeros() as usize
        };
        let piece = 1u64 << piece_index;
        if piece & pinned_pieces == 0u64 {
            let piece_target = if let PieceType::Knight = piece_type {
                knight_attack(piece_index)
            } else if let PieceType::Bishop = piece_type {
                bishop_attack(piece_index, all_pieces)
            } else if let PieceType::Rook = piece_type {
                rook_attack(piece_index, all_pieces)
            } else if let PieceType::Queen = piece_type {
                bishop_attack(piece_index, all_pieces) | rook_attack(piece_index, all_pieces)
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
                    GameMoveType::Capture(find_captured_piece_type(
                        capture_index,
                        enemy_pawns,
                        enemy_knights,
                        enemy_bishops,
                        enemy_rooks,
                        enemy_queens,
                    )),
                    depth,
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
                        depth,
                    );
                }
            }
        }
        piece_board ^= piece;
    }
    stm_haslegalmove
}
#[inline(always)]
pub fn add_promotion_move_to_movelist(
    legal_moves: &mut MoveList,
    from_square: usize,
    to_square: usize,
    move_type: GameMoveType,
    depth: usize,
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
        depth,
    );
    add_move_to_movelist(
        legal_moves,
        from_square,
        to_square,
        PieceType::Pawn,
        new_types.1,
        depth,
    );
    add_move_to_movelist(
        legal_moves,
        from_square,
        to_square,
        PieceType::Pawn,
        new_types.2,
        depth,
    );
    add_move_to_movelist(
        legal_moves,
        from_square,
        to_square,
        PieceType::Pawn,
        new_types.3,
        depth,
    );
}
#[inline(always)]
pub fn add_moves_to_movelist(
    legal_moves: &mut MoveList,
    from_square: usize,
    mut target_board: u64,
    piece_type: PieceType,
    move_type: GameMoveType,
    depth: usize,
) {
    while target_board != 0u64 {
        let target_square = target_board.trailing_zeros() as usize;
        add_move_to_movelist(
            legal_moves,
            from_square,
            target_square,
            piece_type,
            move_type,
            depth,
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
    depth: usize,
) {
    legal_moves.add_move(
        GameMove {
            from: from_square,
            to: to_square,
            move_type,
            piece_type,
        },
        depth,
    );
}
pub struct MoveList {
    pub move_list: [[Option<GameMove>; 128]; 100],
    pub graded_moves: [[Option<GradedMove>; 128]; 100],
    //pub graded_moves: Vec<Vec<Option<GradedMove>>>,
    pub counter: [usize; 100],
}
impl Default for MoveList {
    fn default() -> Self {
        MoveList {
            move_list: unsafe { std::mem::uninitialized() },
            graded_moves: unsafe { std::mem::uninitialized() },
            //move_list: [[None; 128]; 100],
            //graded_moves: [[None; 128]; 100],
            //graded_moves: vec![vec![None; 128]; 100],
            counter: [0; 100],
        }
    }
}

impl MoveList {
    pub fn add_move(&mut self, mv: GameMove, depth: usize) {
        self.move_list[depth][self.counter[depth]] = Some(mv);
        self.counter[depth] += 1;
    }
}
pub fn generate_moves(
    g: &game_state::GameState,
    only_captures: bool,
    movelist: &mut MoveList,
    depth: usize,
) -> AdditionalGameStateInformation {
    //----------------------------------------------------------------------
    //**********************************************************************
    //0. General bitboards and variable initialization
    movelist.counter[depth] = 0;

    let stm_color = g.color_to_move;
    let enemy_color = 1 - stm_color;
    let stm_color_iswhite: bool = g.color_to_move == WHITE;

    let mut stm_pawns: u64 = g.pieces[PAWN][stm_color];
    let stm_knights: u64 = g.pieces[KNIGHT][stm_color];
    let stm_bishops: u64 = g.pieces[BISHOP][stm_color];
    let stm_rooks: u64 = g.pieces[ROOK][stm_color];
    let stm_queens: u64 = g.pieces[QUEEN][stm_color];
    let stm_king: u64 = g.pieces[KING][stm_color];
    let stm_king_index: usize = stm_king.trailing_zeros() as usize;

    let enemy_pawns: u64 = g.pieces[PAWN][enemy_color];
    let enemy_knights: u64 = g.pieces[KNIGHT][enemy_color];
    let enemy_bishops: u64 = g.pieces[BISHOP][enemy_color];
    let enemy_rooks: u64 = g.pieces[ROOK][enemy_color];
    let enemy_queens: u64 = g.pieces[QUEEN][enemy_color];
    let enemy_king: u64 = g.pieces[KING][enemy_color];

    let mut stm_haslegalmove = false;
    //----------------------------------------------------------------------
    //**********************************************************************
    //1. Calculate additional needed bitboards
    let abb = calculate_additionalbitboards(
        stm_pawns,
        enemy_pawns,
        stm_knights,
        enemy_knights,
        stm_bishops,
        enemy_bishops,
        stm_rooks,
        enemy_rooks,
        stm_queens,
        enemy_queens,
        stm_king,
        enemy_king,
        stm_color_iswhite,
    );
    //----------------------------------------------------------------------
    //**********************************************************************
    //2. Safe King moves
    let stm_legal_kingmoves = abb.stm_king_attacks & !abb.stm_unsafe_squares;
    stm_haslegalmove |= stm_legal_kingmoves != 0u64;
    add_king_moves_to_movelist(
        movelist,
        only_captures,
        stm_legal_kingmoves,
        stm_king_index,
        enemy_pawns,
        enemy_knights,
        enemy_bishops,
        enemy_rooks,
        enemy_queens,
        abb.enemy_pieces,
        depth,
    );
    //----------------------------------------------------------------------
    //**********************************************************************
    //3. Check & Check Evasions
    let checkers = abb.all_checkers.count_ones() as usize;
    let stm_incheck = checkers > 0;

    let mut capture_mask = 0xFFFF_FFFF_FFFF_FFFFu64;
    let mut push_mask = 0xFFFF_FFFF_FFFF_FFFFu64;
    if checkers > 1 {
        //Double check, only safe king moves are legal
        return AdditionalGameStateInformation {
            stm_incheck,
            stm_haslegalmove,
            additional_bitboards: abb,
        };
    } else if checkers == 1 {
        //Only a single checker
        capture_mask = abb.all_checkers;
        //If it's a slider, we can also push in its way
        if abb.all_checkers & (enemy_bishops | enemy_rooks | enemy_queens) != 0u64 {
            let checker_square = abb.all_checkers.trailing_zeros() as usize;
            if abb.all_checkers & (bitboards::FREEFIELD_ROOK_ATTACKS[stm_king_index]) != 0u64 {
                //Checker is rook-like
                push_mask = get_rook_ray(stm_king_index, checker_square);
            } else {
                //Checker is bishop-like
                push_mask = get_bishop_ray(stm_king_index, checker_square);
            }
        } else {
            //else, we can't do push (quiet) moves
            push_mask = 0u64;
        }
    }

    let empty_squares = !abb.all_pieces;
    //----------------------------------------------------------------------
    //**********************************************************************
    //4. Pins and pinned pieces
    let mut pinned_pieces = 0u64;
    //4.1 Rook-Like pins
    let stm_rook_attacks_from_king = rook_attack(stm_king_index, abb.all_pieces);
    let stm_xray_rook_attacks_from_king = xray_rook_attacks(
        stm_rook_attacks_from_king,
        abb.all_pieces,
        abb.stm_pieces,
        stm_king_index,
    );
    let mut enemy_rooks_on_xray = stm_xray_rook_attacks_from_king & (enemy_rooks | enemy_queens);
    while enemy_rooks_on_xray != 0u64 {
        let enemy_rook_position = enemy_rooks_on_xray.trailing_zeros() as usize;
        let enemy_rook = 1u64 << enemy_rook_position;
        let ray_to_king = get_rook_ray(stm_king_index, enemy_rook_position);
        let pinned_piece = ray_to_king & abb.stm_pieces;
        let pinned_piece_position = pinned_piece.trailing_zeros() as usize;
        pinned_pieces |= pinned_piece;
        if pinned_piece & stm_queens != 0u64 {
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
                enemy_queens,
                PieceType::Rook,
                depth,
            );
        } else if pinned_piece & stm_rooks != 0u64 {
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
                enemy_queens,
                PieceType::Rook,
                depth,
            );
        } else if pinned_piece & stm_pawns != 0u64 {
            //Add possible pawn pushes
            stm_pawns ^= pinned_piece;
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
                    depth,
                )
            }
        }
        enemy_rooks_on_xray ^= enemy_rook;
    }
    //4.2 Bishop-Like pins
    let stm_bishop_attacks_from_king = bishop_attack(stm_king_index, abb.all_pieces);
    let stm_xray_bishop_attacks_from_king = xray_bishop_attacks(
        stm_bishop_attacks_from_king,
        abb.all_pieces,
        abb.stm_pieces,
        stm_king_index,
    );
    let mut enemy_bishop_on_xray =
        stm_xray_bishop_attacks_from_king & (enemy_bishops | enemy_queens);
    while enemy_bishop_on_xray != 0u64 {
        let enemy_bishop_position = enemy_bishop_on_xray.trailing_zeros() as usize;
        let enemy_bishop = 1u64 << enemy_bishop_position;
        let ray_to_king = get_bishop_ray(stm_king_index, enemy_bishop_position);
        let pinned_piece = ray_to_king & abb.stm_pieces;
        let pinned_piece_position = pinned_piece.trailing_zeros() as usize;
        pinned_pieces |= pinned_piece;
        if pinned_piece & stm_queens != 0u64 {
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
                enemy_queens,
                PieceType::Bishop,
                depth,
            );
        } else if pinned_piece & stm_bishops != 0u64 {
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
                enemy_queens,
                PieceType::Bishop,
                depth,
            );
        } else if pinned_piece & stm_pawns != 0u64 {
            //Add possible pawn captures
            stm_pawns ^= pinned_piece;

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
                    GameMoveType::Capture(if enemy_bishop & enemy_queens != 0u64 {
                        PieceType::Queen
                    } else {
                        PieceType::Bishop
                    }),
                    depth,
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
                    GameMoveType::Capture(if enemy_bishop & enemy_queens != 0u64 {
                        PieceType::Queen
                    } else {
                        PieceType::Bishop
                    }),
                    depth,
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
                    depth,
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
        w_single_push_pawn_targets(stm_pawns, empty_squares)
    } else {
        b_single_push_pawn_targets(stm_pawns, empty_squares)
    } & push_mask;
    stm_haslegalmove |= stm_pawns_single_push != 0u64;
    let stm_pawn_promotions =
        stm_pawns_single_push & bitboards::RANKS[if stm_color_iswhite { 7 } else { 0 }];
    if !only_captures {
        add_pawn_moves_to_movelist(
            movelist,
            stm_pawn_promotions,
            8,
            stm_color_iswhite,
            enemy_pawns,
            enemy_knights,
            enemy_bishops,
            enemy_rooks,
            enemy_queens,
            false,
            true,
            pinned_pieces,
            depth,
        );
    }
    if !only_captures {
        let stm_pawns_quiet_single_push = stm_pawns_single_push & !stm_pawn_promotions;
        add_pawn_moves_to_movelist(
            movelist,
            stm_pawns_quiet_single_push,
            8,
            stm_color_iswhite,
            enemy_pawns,
            enemy_knights,
            enemy_bishops,
            enemy_rooks,
            enemy_queens,
            false,
            false,
            pinned_pieces,
            depth,
        );
    }
    //5.2 Double push
    if !only_captures || !stm_haslegalmove {
        let stm_pawns_double_push = if stm_color_iswhite {
            w_double_push_pawn_targets(stm_pawns, empty_squares)
        } else {
            b_double_push_pawn_targets(stm_pawns, empty_squares)
        } & push_mask;
        stm_haslegalmove |= stm_pawns_double_push != 0u64;
        if !only_captures {
            add_pawn_moves_to_movelist(
                movelist,
                stm_pawns_double_push,
                16,
                stm_color_iswhite,
                enemy_pawns,
                enemy_knights,
                enemy_bishops,
                enemy_rooks,
                enemy_queens,
                false,
                false,
                pinned_pieces,
                depth,
            );
        }
    }
    //5.3 West captures (normal capture, promotion capture, en passant)
    let stm_pawn_west_captures = abb.stm_pawns_westattack & capture_mask & abb.enemy_pieces;
    //Split up in promotion and non-promotion captures
    let stm_pawn_west_promotion_capture =
        stm_pawn_west_captures & bitboards::RANKS[if stm_color_iswhite { 7 } else { 0 }];
    stm_haslegalmove |= add_pawn_moves_to_movelist(
        movelist,
        stm_pawn_west_promotion_capture,
        7,
        stm_color_iswhite,
        enemy_pawns,
        enemy_knights,
        enemy_bishops,
        enemy_rooks,
        enemy_queens,
        true,
        true,
        pinned_pieces,
        depth,
    );
    let stm_pawn_west_nonpromotion_capture =
        stm_pawn_west_captures & !stm_pawn_west_promotion_capture;
    stm_haslegalmove |= add_pawn_moves_to_movelist(
        movelist,
        stm_pawn_west_nonpromotion_capture,
        7,
        stm_color_iswhite,
        enemy_pawns,
        enemy_knights,
        enemy_bishops,
        enemy_rooks,
        enemy_queens,
        true,
        false,
        pinned_pieces,
        depth,
    );
    //En passants
    let stm_pawn_west_enpassants = abb.stm_pawns_westattack
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
            abb.all_pieces & !(1u64 << pawn_from) & !(1u64 << removed_piece_index);
        if rook_attack(stm_king_index, all_pieces_without_en_passants)
            & bitboards::RANKS[stm_king_index / 8]
            & (enemy_rooks | enemy_queens)
            == 0u64
        {
            stm_haslegalmove = true;
            add_move_to_movelist(
                movelist,
                pawn_from,
                pawn_index,
                PieceType::Pawn,
                GameMoveType::EnPassant,
                depth,
            );
        }
    }
    //5.4 East captures (normal capture, promotion capture, en passant)
    let stm_pawn_east_captures = abb.stm_pawns_eastattack & capture_mask & abb.enemy_pieces;
    //Split up in promotion and non-promotion captures
    let stm_pawn_east_promotion_capture =
        stm_pawn_east_captures & bitboards::RANKS[if stm_color_iswhite { 7 } else { 0 }];
    stm_haslegalmove |= add_pawn_moves_to_movelist(
        movelist,
        stm_pawn_east_promotion_capture,
        9,
        stm_color_iswhite,
        enemy_pawns,
        enemy_knights,
        enemy_bishops,
        enemy_rooks,
        enemy_queens,
        true,
        true,
        pinned_pieces,
        depth,
    );
    let stm_pawn_east_nonpromotion_capture =
        stm_pawn_east_captures & !stm_pawn_east_promotion_capture;
    stm_haslegalmove |= add_pawn_moves_to_movelist(
        movelist,
        stm_pawn_east_nonpromotion_capture,
        9,
        stm_color_iswhite,
        enemy_pawns,
        enemy_knights,
        enemy_bishops,
        enemy_rooks,
        enemy_queens,
        true,
        false,
        pinned_pieces,
        depth,
    );
    //En passants
    let stm_pawn_east_enpassants = abb.stm_pawns_eastattack
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
            abb.all_pieces & !(1u64 << pawn_from) & !(1u64 << removed_piece_index);
        if rook_attack(stm_king_index, all_pieces_without_en_passants)
            & bitboards::RANKS[stm_king_index / 8]
            & (enemy_rooks | enemy_queens)
            == 0u64
        {
            stm_haslegalmove = true;
            add_move_to_movelist(
                movelist,
                pawn_from,
                pawn_index,
                PieceType::Pawn,
                GameMoveType::EnPassant,
                depth,
            );
        }
    }

    //----------------------------------------------------------------------
    //**********************************************************************
    //6. All other legal moves (knights, bishops, rooks, queens)
    //6.1 Knights
    stm_haslegalmove |= add_normal_moves_to_movelist(
        movelist,
        PieceType::Knight,
        stm_knights,
        pinned_pieces,
        enemy_pawns,
        enemy_knights,
        enemy_bishops,
        enemy_rooks,
        enemy_queens,
        abb.enemy_pieces,
        empty_squares,
        abb.all_pieces,
        push_mask,
        capture_mask,
        only_captures,
        depth,
        stm_color_iswhite,
    );
    //6.4 Queens
    stm_haslegalmove |= add_normal_moves_to_movelist(
        movelist,
        PieceType::Queen,
        stm_queens,
        pinned_pieces,
        enemy_pawns,
        enemy_knights,
        enemy_bishops,
        enemy_rooks,
        enemy_queens,
        abb.enemy_pieces,
        empty_squares,
        abb.all_pieces,
        push_mask,
        capture_mask,
        only_captures,
        depth,
        stm_color_iswhite,
    );

    //6.2 Bishops
    stm_haslegalmove |= add_normal_moves_to_movelist(
        movelist,
        PieceType::Bishop,
        stm_bishops,
        pinned_pieces,
        enemy_pawns,
        enemy_knights,
        enemy_bishops,
        enemy_rooks,
        enemy_queens,
        abb.enemy_pieces,
        empty_squares,
        abb.all_pieces,
        push_mask,
        capture_mask,
        only_captures,
        depth,
        stm_color_iswhite,
    );
    //6.3 Rooks
    stm_haslegalmove |= add_normal_moves_to_movelist(
        movelist,
        PieceType::Rook,
        stm_rooks,
        pinned_pieces,
        enemy_pawns,
        enemy_knights,
        enemy_bishops,
        enemy_rooks,
        enemy_queens,
        abb.enemy_pieces,
        empty_squares,
        abb.all_pieces,
        push_mask,
        capture_mask,
        only_captures,
        depth,
        stm_color_iswhite,
    );
    //----------------------------------------------------------------------
    //**********************************************************************
    //7. Castling
    if (!only_captures || !stm_haslegalmove) && checkers == 0 {
        if stm_color_iswhite {
            if g.castle_white_kingside
                && (abb.all_pieces | abb.stm_unsafe_squares)
                    & (bitboards::SQUARES[5] | bitboards::SQUARES[6])
                    == 0u64
            {
                stm_haslegalmove = true;
                if !only_captures {
                    movelist.add_move(
                        GameMove {
                            from: stm_king_index,
                            to: 6usize,
                            move_type: GameMoveType::Castle,
                            piece_type: PieceType::King,
                        },
                        depth,
                    );
                }
            }
            if g.castle_white_queenside
                && ((abb.all_pieces | abb.stm_unsafe_squares)
                    & (bitboards::SQUARES[2] | bitboards::SQUARES[3])
                    | abb.all_pieces & bitboards::SQUARES[1])
                    == 0u64
            {
                stm_haslegalmove = true;
                if !only_captures {
                    movelist.add_move(
                        GameMove {
                            from: stm_king_index,
                            to: 2usize,
                            move_type: GameMoveType::Castle,
                            piece_type: PieceType::King,
                        },
                        depth,
                    );
                }
            }
        } else {
            if g.castle_black_kingside
                && (abb.all_pieces | abb.stm_unsafe_squares)
                    & (bitboards::SQUARES[61] | bitboards::SQUARES[62])
                    == 0u64
            {
                stm_haslegalmove = true;
                if !only_captures {
                    movelist.add_move(
                        GameMove {
                            from: stm_king_index,
                            to: 62usize,
                            move_type: GameMoveType::Castle,
                            piece_type: PieceType::King,
                        },
                        depth,
                    );
                }
            }
            if g.castle_black_queenside
                && ((abb.all_pieces | abb.stm_unsafe_squares)
                    & (bitboards::SQUARES[58] | bitboards::SQUARES[59])
                    | abb.all_pieces & bitboards::SQUARES[57])
                    == 0u64
            {
                stm_haslegalmove = true;
                if !only_captures {
                    movelist.add_move(
                        GameMove {
                            from: stm_king_index,
                            to: 58usize,
                            move_type: GameMoveType::Castle,
                            piece_type: PieceType::King,
                        },
                        depth,
                    );
                }
            }
        }
    }
    //----------------------------------------------------------------------
    AdditionalGameStateInformation {
        stm_incheck,
        stm_haslegalmove,
        additional_bitboards: abb,
    }
}
