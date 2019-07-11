use super::super::bitboards;
use super::super::board_representation::game_state::{self, GameMove, GameMoveType, PieceType};
use super::magic::{self, Magic};
use crate::search::GradedMove;

//Move GEn
//King- Piece-Wise by lookup
//Knight-Piece-Wise by lookup
//Bishop/Queen/Rook - Piece-Wise by lookup in Magic
//Pawn-SetWise by shift
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
#[inline(always)]
pub fn w_single_push_pawn_targets(pawns: u64, empty: u64) -> u64 {
    bitboards::north_one(pawns) & empty
}
#[inline(always)]
pub fn w_double_push_pawn_targets(pawns: u64, empty: u64) -> u64 {
    bitboards::north_one(bitboards::north_one(pawns & bitboards::RANKS[1]) & empty) & empty
}
#[inline(always)]
pub fn b_single_push_pawn_targets(pawns: u64, empty: u64) -> u64 {
    bitboards::south_one(pawns) & empty
}
#[inline(always)]
pub fn b_double_push_pawn_targets(pawns: u64, empty: u64) -> u64 {
    bitboards::south_one(bitboards::south_one(pawns & bitboards::RANKS[6]) & empty) & empty
}

//NorthEast = +9
#[inline(always)]
pub fn w_pawn_east_targets(pawns: u64) -> u64 {
    bitboards::north_east_one(pawns)
}

//NorthWest = +7
#[inline(always)]
pub fn w_pawn_west_targets(pawns: u64) -> u64 {
    bitboards::north_west_one(pawns)
}

//SouthEast = -7
#[inline(always)]
pub fn b_pawn_east_targets(pawns: u64) -> u64 {
    bitboards::south_west_one(pawns)
}

//NorthWest = -9
#[inline(always)]
pub fn b_pawn_west_targets(pawns: u64) -> u64 {
    bitboards::south_east_one(pawns)
}

#[inline(always)]
pub fn add_moves(
    move_list: &mut Vec<GameMove>,
    from: usize,
    mut to_board: u64,
    piece_type: &PieceType,
    move_type: GameMoveType,
) {
    while to_board != 0u64 {
        let idx = to_board.trailing_zeros() as usize;
        let move_t_cl = move_type.clone();
        let pt_cl = piece_type.clone();
        move_list.push(GameMove {
            from,
            to: idx,
            move_type: move_t_cl,
            piece_type: pt_cl,
        });
        to_board ^= 1u64 << idx;
        //to_board&= to_board-1;
    }
}

#[inline(always)]
pub fn add_capture_moves(
    move_list: &mut Vec<GameMove>,
    from: usize,
    mut to_board: u64,
    piece_type: &PieceType,
    enemy_pawns: u64,
    enemy_knights: u64,
    enemy_bishops: u64,
    enemy_rooks: u64,
    enemy_queens: u64,
) {
    while to_board != 0u64 {
        let idx = to_board.trailing_zeros() as usize;
        let pt_cl = piece_type.clone();
        move_list.push(GameMove {
            from,
            to: idx,
            move_type: GameMoveType::Capture(find_captured_piece_type(
                idx,
                enemy_pawns,
                enemy_knights,
                enemy_bishops,
                enemy_rooks,
                enemy_queens,
            )),
            piece_type: pt_cl,
        });
        to_board ^= 1u64 << idx;
    }
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
    //Find queens before bishops and rooks since it's in all three boards.
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
pub fn add_pawn_single_pushes(
    mut single_push_board: u64,
    color_to_move: &usize,
    move_list: &mut Vec<GameMove>,
) {
    while single_push_board != 0u64 {
        let idx = single_push_board.trailing_zeros() as usize;
        move_list.push(GameMove {
            from: if *color_to_move == 0 {
                idx - 8
            } else {
                idx + 8
            },
            to: idx,
            move_type: GameMoveType::Quiet,
            piece_type: PieceType::Pawn,
        });
        single_push_board ^= 1 << idx;
    }
}

#[inline(always)]
pub fn add_pawn_double_pushes(
    mut double_push_board: u64,
    color_to_move: &usize,
    move_list: &mut Vec<GameMove>,
) {
    while double_push_board != 0u64 {
        let idx = double_push_board.trailing_zeros() as usize;
        move_list.push(GameMove {
            from: if *color_to_move == 0 {
                idx - 16
            } else {
                idx + 16
            },
            to: idx,
            move_type: GameMoveType::Quiet,
            piece_type: PieceType::Pawn,
        });
        double_push_board ^= 1 << idx;
    }
}

#[inline(always)]
pub fn add_promotion_push(
    mut promotion_board: u64,
    color_to_move: &usize,
    move_list: &mut Vec<GameMove>,
) {
    while promotion_board != 0u64 {
        let idx = promotion_board.trailing_zeros() as usize;
        move_list.push(GameMove {
            from: if *color_to_move == 0 {
                idx - 8
            } else {
                idx + 8
            },
            to: idx,
            move_type: GameMoveType::Promotion(PieceType::Queen, None),
            piece_type: PieceType::Pawn,
        });
        move_list.push(GameMove {
            from: if *color_to_move == 0 {
                idx - 8
            } else {
                idx + 8
            },
            to: idx,
            move_type: GameMoveType::Promotion(PieceType::Rook, None),
            piece_type: PieceType::Pawn,
        });
        move_list.push(GameMove {
            from: if *color_to_move == 0 {
                idx - 8
            } else {
                idx + 8
            },
            to: idx,
            move_type: GameMoveType::Promotion(PieceType::Bishop, None),
            piece_type: PieceType::Pawn,
        });
        move_list.push(GameMove {
            from: if *color_to_move == 0 {
                idx - 8
            } else {
                idx + 8
            },
            to: idx,
            move_type: GameMoveType::Promotion(PieceType::Knight, None),
            piece_type: PieceType::Pawn,
        });
        promotion_board ^= 1 << idx;
    }
}

#[inline(always)]
pub fn add_promotion_capture(
    mut promotion_board: u64,
    color_to_move: &usize,
    move_list: &mut Vec<GameMove>,
    source_shift: usize,
    enemy_pawns: u64,
    enemy_knights: u64,
    enemy_bishops: u64,
    enemy_rooks: u64,
    enemy_queens: u64,
) {
    while promotion_board != 0u64 {
        let idx = promotion_board.trailing_zeros() as usize;
        let x: Option<PieceType> = Some(find_captured_piece_type(
            idx,
            enemy_pawns,
            enemy_knights,
            enemy_bishops,
            enemy_rooks,
            enemy_queens,
        ));
        move_list.push(GameMove {
            from: if *color_to_move == 0 {
                idx - source_shift
            } else {
                idx + source_shift
            },
            to: idx,
            move_type: GameMoveType::Promotion(PieceType::Queen, x.clone()),
            piece_type: PieceType::Pawn,
        });
        move_list.push(GameMove {
            from: if *color_to_move == 0 {
                idx - source_shift
            } else {
                idx + source_shift
            },
            to: idx,
            move_type: GameMoveType::Promotion(PieceType::Rook, x.clone()),
            piece_type: PieceType::Pawn,
        });
        move_list.push(GameMove {
            from: if *color_to_move == 0 {
                idx - source_shift
            } else {
                idx + source_shift
            },
            to: idx,
            move_type: GameMoveType::Promotion(PieceType::Bishop, x.clone()),
            piece_type: PieceType::Pawn,
        });
        move_list.push(GameMove {
            from: if *color_to_move == 0 {
                idx - source_shift
            } else {
                idx + source_shift
            },
            to: idx,
            move_type: GameMoveType::Promotion(PieceType::Knight, x),
            piece_type: PieceType::Pawn,
        });
        promotion_board ^= 1 << idx;
    }
}

#[inline(always)]
pub fn add_pawn_capture(
    mut capture_board: u64,
    color_to_move: &usize,
    move_list: &mut Vec<GameMove>,
    source_shift: usize,
    enemy_pawns: u64,
    enemy_knights: u64,
    enemy_bishops: u64,
    enemy_rooks: u64,
    enemy_queens: u64,
) {
    while capture_board != 0u64 {
        let idx = capture_board.trailing_zeros() as usize;
        move_list.push(GameMove {
            from: if *color_to_move == 0 {
                idx - source_shift
            } else {
                idx + source_shift
            },
            to: idx,
            move_type: GameMoveType::Capture(find_captured_piece_type(
                idx,
                enemy_pawns,
                enemy_knights,
                enemy_bishops,
                enemy_rooks,
                enemy_queens,
            )),
            piece_type: PieceType::Pawn,
        });
        capture_board ^= 1 << idx;
    }
}

#[inline(always)]
pub fn add_en_passants(
    mut enpassant_board: u64,
    color_to_move: &usize,
    move_list: &mut Vec<GameMove>,
    source_shift: usize,
    all_pieces_without_my_king: u64,
    enemy_rooks: u64,
    my_king_idx: usize,
) {
    while enpassant_board != 0u64 {
        let index = enpassant_board.trailing_zeros() as usize;
        enpassant_board ^= 1u64 << index;
        //Check if rare case didn't happen
        //Remove t-7,t-8 or t+7,t+8
        let all_pieces_without_en_passants = all_pieces_without_my_king
            & !bitboards::SQUARES[if *color_to_move == 0 {
                index - source_shift
            } else {
                index + source_shift
            }]
            & !bitboards::SQUARES[if *color_to_move == 0 {
                index - 8
            } else {
                index + 8
            }];
        if rook_attack(my_king_idx, all_pieces_without_en_passants)
            & (!bitboards::FILES[my_king_idx % 8])
            & enemy_rooks
            != 0
        {
            continue;
        }
        move_list.push(GameMove {
            from: if *color_to_move == 0 {
                index - source_shift
            } else {
                index + source_shift
            },
            to: index,
            move_type: GameMoveType::EnPassant,
            piece_type: PieceType::Pawn,
        });
    }
}

#[inline(always)]
pub fn xray_rook_attacks(
    rook_attacks: u64,
    occupied_squares: u64,
    my_pieces: u64,
    rook_square: usize,
) -> u64 {
    return rook_attacks ^ rook_attack(rook_square, occupied_squares ^ (my_pieces & rook_attacks));
}
#[inline(always)]
pub fn xray_bishop_attacks(
    bishop_attacks: u64,
    occupied_squares: u64,
    my_pieces: u64,
    bishop_square: usize,
) -> u64 {
    return bishop_attacks
        ^ bishop_attack(
            bishop_square,
            occupied_squares ^ (my_pieces & bishop_attacks),
        );
}
#[inline(always)]
pub fn get_rook_ray(king_square: usize, rook_square: usize) -> u64 {
    bitboards::ROOK_RAYS[king_square][rook_square]
}

#[inline(always)]
pub fn get_bishop_ray(king_square: usize, bishop_square: usize) -> u64 {
    bitboards::BISHOP_RAYS[king_square][bishop_square]
}

pub fn attackers_from_white(
    square_board: u64,
    square: usize,
    white_pawns: u64,
    white_knights: u64,
    white_bishops: u64,
    white_rooks: u64,
    blockers: u64,
) -> (u64, bool, bool) {
    let mut attackers = 0u64;
    let mut slider_flag = false;
    let mut bishop_slider = false;
    attackers |= knight_attack(square) & white_knights;
    attackers |=
        (b_pawn_west_targets(square_board) | b_pawn_east_targets(square_board)) & white_pawns;
    let bishop_attacks = bishop_attack(square, blockers) & white_bishops;
    attackers |= bishop_attacks;
    if bishop_attacks != 0 {
        slider_flag = true;
        bishop_slider = true;
    }
    let rook_attacks = rook_attack(square, blockers) & white_rooks;
    attackers |= rook_attacks;
    if rook_attacks != 0 {
        slider_flag = true;
    }
    (attackers, slider_flag, bishop_slider)
}

pub fn attackers_from_black(
    square_board: u64,
    square: usize,
    black_pawns: u64,
    black_knights: u64,
    black_bishops: u64,
    black_rooks: u64,
    blockers: u64,
) -> (u64, bool, bool) {
    let mut attackers = 0u64;
    let mut slider_flag = false;
    let mut bishop_slider = false;
    attackers |= knight_attack(square) & black_knights;
    attackers |=
        (w_pawn_west_targets(square_board) | w_pawn_east_targets(square_board)) & black_pawns;
    let bishop_attacks = bishop_attack(square, blockers) & black_bishops;
    attackers |= bishop_attacks;
    if bishop_attacks != 0 {
        slider_flag = true;
        bishop_slider = true;
    }
    let rook_attacks = rook_attack(square, blockers) & black_rooks;
    attackers |= rook_attacks;
    if rook_attacks != 0 {
        slider_flag = true;
    }
    (attackers, slider_flag, bishop_slider)
}

pub fn get_w_attacked_squares(
    white_king_idx: usize,
    white_pawns: u64,
    mut white_knights: u64,
    mut white_bishops: u64,
    mut white_rooks: u64,
    blocked_squares: u64,
) -> u64 {
    let mut res = 0u64;
    res |= king_attack(white_king_idx);
    res |= w_pawn_west_targets(white_pawns) | w_pawn_east_targets(white_pawns);
    while white_knights != 0u64 {
        let sq = 63usize - white_knights.leading_zeros() as usize;
        res |= knight_attack(sq);
        white_knights ^= 1u64 << sq;
    }
    while white_bishops != 0u64 {
        let sq = 63usize - white_bishops.leading_zeros() as usize;
        res |= bishop_attack(sq, blocked_squares);
        white_bishops ^= 1u64 << sq;
    }
    while white_rooks != 0u64 {
        let sq = 63usize - white_rooks.leading_zeros() as usize;
        res |= rook_attack(sq, blocked_squares);
        white_rooks ^= 1u64 << sq;
    }
    res
}

pub fn get_b_attacked_squares(
    black_king_idx: usize,
    black_pawns: u64,
    mut black_knights: u64,
    mut black_bishops: u64,
    mut black_rooks: u64,
    blocked_squares: u64,
) -> u64 {
    let mut res = 0u64;
    res |= king_attack(black_king_idx);
    res |= b_pawn_west_targets(black_pawns) | b_pawn_east_targets(black_pawns);
    while black_knights != 0u64 {
        let sq = black_knights.trailing_zeros() as usize;
        res |= knight_attack(sq);
        black_knights ^= 1u64 << sq;
    }
    while black_bishops != 0u64 {
        let sq = black_bishops.trailing_zeros() as usize;
        res |= bishop_attack(sq, blocked_squares);
        black_bishops ^= 1u64 << sq;
    }
    while black_rooks != 0u64 {
        let sq = black_rooks.trailing_zeros() as usize;
        res |= rook_attack(sq, blocked_squares);
        black_rooks ^= 1u64 << sq;
    }
    res
}

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

/*#[inline(always)]
pub fn calculate_additionalbitboards_g(
    g: &game_state::GameState,
) -> AdditionalBitBoards {
    let stm_color = g.color_to_move;
    let enemy_color = 1 - stm_color;
    let stm_color_iswhite: bool = g.color_to_move == 0;

    let stm_pawns: u64 = g.pieces[0][stm_color];
    let stm_knights: u64 = g.pieces[1][stm_color];
    let stm_bishops: u64 = g.pieces[2][stm_color];
    let stm_rooks: u64 = g.pieces[3][stm_color];
    let stm_queens: u64 = g.pieces[4][stm_color];
    let stm_king: u64 = g.pieces[5][stm_color];

    let enemy_pawns: u64 = g.pieces[0][enemy_color];
    let enemy_knights: u64 = g.pieces[1][enemy_color];
    let enemy_bishops: u64 = g.pieces[2][enemy_color];
    let enemy_rooks: u64 = g.pieces[3][enemy_color];
    let enemy_queens: u64 = g.pieces[4][enemy_color];
    let enemy_king: u64 = g.pieces[5][enemy_color];
    calculate_additionalbitboards(
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
        stm_color_iswhite
    )
}*/

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
    let (
        stm_pawns_westattack,
        stm_pawns_eastattack,
        enemy_pawns_westattack,
        enemy_pawns_eastattack,
    ) = if stm_color_iswhite {
        (
            w_pawn_west_targets(stm_pawns),
            w_pawn_east_targets(stm_pawns),
            b_pawn_west_targets(enemy_pawns),
            b_pawn_east_targets(enemy_pawns),
        )
    } else {
        (
            b_pawn_west_targets(stm_pawns),
            b_pawn_east_targets(stm_pawns),
            w_pawn_west_targets(enemy_pawns),
            w_pawn_east_targets(enemy_pawns),
        )
    };

    stm_unsafe_squares |= enemy_pawns_westattack | enemy_pawns_eastattack;
    all_checkers |= if stm_color_iswhite {
        w_pawn_west_targets(stm_king & enemy_pawns_westattack)
    } else {
        b_pawn_west_targets(stm_king & enemy_pawns_westattack)
    } | if stm_color_iswhite {
        w_pawn_east_targets(stm_king & enemy_pawns_eastattack)
    } else {
        b_pawn_east_targets(stm_king & enemy_pawns_eastattack)
    };

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
            moving_piece_type.clone(),
            GameMoveType::Quiet,
            depth,
        );
    }
    if pin_capture_possible {
        add_move_to_movelist(
            legal_moves,
            pinned_piece_position,
            pinner_position,
            moving_piece_type.clone(),
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
    let mut index = 0;
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
                    piece_type.clone(),
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
                        piece_type.clone(),
                        GameMoveType::Quiet,
                        depth,
                    );
                }
            }
        }
        index = index + 1;
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
            GameMoveType::Promotion(PieceType::Queen, Some(x.clone())),
            GameMoveType::Promotion(PieceType::Rook, Some(x.clone())),
            GameMoveType::Promotion(PieceType::Bishop, Some(x.clone())),
            GameMoveType::Promotion(PieceType::Knight, Some(x.clone())),
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
            piece_type.clone(),
            move_type.clone(),
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
            move_type: move_type,
            piece_type: piece_type,
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
impl MoveList {
    pub fn new() -> Self {
        MoveList {
            move_list: unsafe { std::mem::uninitialized() },
            graded_moves: unsafe { std::mem::uninitialized() },
            //move_list: [[None; 128]; 100],
            //graded_moves: [[None; 128]; 100],
            //graded_moves: vec![vec![None; 128]; 100],
            counter: [0; 100],
        }
    }
    pub fn add_move(&mut self, mv: GameMove, depth: usize) {
        self.move_list[depth][self.counter[depth]] = Some(mv);
        self.counter[depth] += 1;
    }
}
pub fn generate_moves2(
    g: &game_state::GameState,
    only_captures: bool,
    movelist: &mut MoveList,
    depth: usize,
) -> AdditionalGameStateInformation {
    //**********************************************************************
    //0.General Bitboards and Variable Initialization
    movelist.counter[depth] = 0;

    let stm_color = g.color_to_move;
    let enemy_color = 1 - stm_color;
    let stm_color_iswhite: bool = g.color_to_move == 0;

    let mut stm_pawns: u64 = g.pieces[0][stm_color];
    let stm_knights: u64 = g.pieces[1][stm_color];
    let stm_bishops: u64 = g.pieces[2][stm_color];
    let stm_rooks: u64 = g.pieces[3][stm_color];
    let stm_queens: u64 = g.pieces[4][stm_color];
    let stm_king: u64 = g.pieces[5][stm_color];
    let stm_king_index: usize = stm_king.trailing_zeros() as usize;

    let enemy_pawns: u64 = g.pieces[0][enemy_color];
    let enemy_knights: u64 = g.pieces[1][enemy_color];
    let enemy_bishops: u64 = g.pieces[2][enemy_color];
    let enemy_rooks: u64 = g.pieces[3][enemy_color];
    let enemy_queens: u64 = g.pieces[4][enemy_color];
    let enemy_king: u64 = g.pieces[5][enemy_color];

    let mut stm_haslegalmove = false;
    //----------------------------------------------------------------------
    //**********************************************************************
    //1.Calculate additional needed bitboards
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

    let mut capture_mask = 0xFFFFFFFFFFFFFFFFu64;
    let mut push_mask = 0xFFFFFFFFFFFFFFFFu64;
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
        //If it's a slider, we can also push in it's way
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
            //En-Passants
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
    //5. Pawn pushes, captures, and promotions (captures, capture-enpassant,capture-promotion,normal-promotion)
    //5.1 Single push (Promotions and pushes)
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
    //5.3 West captures (normal capture, promotion capture, en-passant)
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
    //En-Passants
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
    //5.4 East captures (normal capture, promotion capture, en-passant)
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
    //En-Passants
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
    //6. All other legal moves (knights,bishops,rooks,queens)
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
            if g.castle_white_kingside {
                if (abb.all_pieces | abb.stm_unsafe_squares)
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
            }
            if g.castle_white_queenside {
                if ((abb.all_pieces | abb.stm_unsafe_squares)
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
            }
        } else {
            if g.castle_black_kingside {
                if (abb.all_pieces | abb.stm_unsafe_squares)
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
            }
            if g.castle_black_queenside {
                if ((abb.all_pieces | abb.stm_unsafe_squares)
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
    }
    //----------------------------------------------------------------------
    let agi = AdditionalGameStateInformation {
        stm_incheck,
        stm_haslegalmove,
        additional_bitboards: abb,
    };
    agi
}
