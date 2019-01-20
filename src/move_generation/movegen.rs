use super::super::bitboards;
use super::super::board_representation::game_state::{self, GameMove, GameMoveType, PieceType};
use super::magic::{self,Magic};
//Move GEn
//King- Piece-Wise by lookup
//Knight-Piece-Wise by lookup
//Bishop/Queen/Rook - Piece-Wise by lookup in Magic
//Pawn-SetWise by shift
pub fn king_attack(square: usize) -> u64 {
    bitboards::KING_ATTACKS[square]
}

pub fn bishop_attack(square: usize, all_pieces: u64) -> u64 {
    Magic::get_attacks(&magic::MAGICS_BISHOPS[square], all_pieces)
}

pub fn rook_attack(square: usize, all_pieces: u64) -> u64 {
    Magic::get_attacks(&magic::MAGICS_ROOKS[square], all_pieces)
}

pub fn knight_attack(square: usize) -> u64 {
    bitboards::KNIGHT_ATTACKS[square]
}

pub fn w_single_push_pawn_targets(pawns: u64, empty: u64) -> u64 {
    bitboards::north_one(pawns) & empty
}

pub fn w_double_push_pawn_targets(pawns: u64, empty: u64) -> u64 {
    bitboards::north_one(bitboards::north_one(pawns & bitboards::RANKS[1]) & empty) & empty
}

pub fn b_single_push_pawn_targets(pawns: u64, empty: u64) -> u64 {
    bitboards::south_one(pawns) & empty
}

pub fn b_double_push_pawn_targets(pawns: u64, empty: u64) -> u64 {
    bitboards::south_one(bitboards::south_one(pawns & bitboards::RANKS[6]) & empty) & empty
}

//NorthEast = +9
pub fn w_pawn_east_targets(pawns: u64) -> u64 {
    bitboards::north_east_one(pawns)
}

//NorthWest = +7
pub fn w_pawn_west_targets(pawns: u64) -> u64 {
    bitboards::north_west_one(pawns)
}

//SouthEast = -7
pub fn b_pawn_east_targets(pawns: u64) -> u64 {
    bitboards::south_west_one(pawns)
}

//NorthWest = -9
pub fn b_pawn_west_targets(pawns: u64) -> u64 {
    bitboards::south_east_one(pawns)
}

#[inline(always)]
pub fn add_moves(move_list: &mut Vec<GameMove>, from: usize, mut to_board: u64, piece_type: &PieceType, move_type: GameMoveType) {
    while to_board != 0u64 {
        let idx = to_board.trailing_zeros() as usize;
        let  move_t_cl = move_type.clone();
        let pt_cl= piece_type.clone();
        move_list.push(GameMove {
            from,
            to: idx,
            move_type: move_t_cl,
            piece_type: pt_cl
        });
        to_board ^= 1u64 << idx;
        //to_board&= to_board-1;
    }
}

//Make moves
pub fn make_move(g: &game_state::GameState, mv: &game_state::GameMove) -> game_state::GameState {
    match mv.move_type {
        GameMoveType::Quiet => make_quiet_move(&g, &mv),
        GameMoveType::Capture => make_capture_move(&g, &mv),
        GameMoveType::EnPassant => make_enpassant_move(&g, &mv),
        GameMoveType::Castle => make_castle_move(&g, &mv),
        GameMoveType::Promotion(PieceType::Queen) | GameMoveType::Promotion(PieceType::Rook) | GameMoveType::Promotion(PieceType::Bishop) | GameMoveType::Promotion(PieceType::Knight)
        => make_promotion_move(&g, &mv),
        _ => panic!("Invalid move type")
    }
}

pub fn move_piece(pieces: &mut [[u64; 2]; 6], mv: &game_state::GameMove, move_color: usize) {
    let index = match mv.piece_type {
        PieceType::Pawn => 0,
        PieceType::Knight => 1,
        PieceType::Bishop => 2,
        PieceType::Rook => 3,
        PieceType::Queen => 4,
        PieceType::King => 5,
    };
    pieces[index][move_color] ^= bitboards::SQUARES[mv.from];
    pieces[index][move_color] |= bitboards::SQUARES[mv.to];
    //pieces[index][move_color] ^= 1u64<<mv.from;
    //pieces[index][move_color] |= 1u64<<mv.to;
}

pub fn delete_piece(pieces: &mut [[u64; 2]; 6], delete_square: usize, delete_color: usize) {
    pieces[0][delete_color] &= bitboards::NOT_SQUARES[delete_square];
    pieces[1][delete_color] &= bitboards::NOT_SQUARES[delete_square];
    pieces[2][delete_color] &= bitboards::NOT_SQUARES[delete_square];
    pieces[3][delete_color] &= bitboards::NOT_SQUARES[delete_square];
    pieces[4][delete_color] &= bitboards::NOT_SQUARES[delete_square];
}

pub fn check_castle_flags(ck: bool, cq: bool, mv: &game_state::GameMove, color_to_move: usize, pieces: [[u64; 2]; 6]) -> (bool, bool) {
    match mv.piece_type {
        PieceType::King => (false, false),
        PieceType::Rook => {
            let mut new_ck = ck;
            if ck {
                if color_to_move == 0 {
                    if pieces[3][0] & bitboards::SQUARES[7] == 0 {
                        new_ck = false;
                    }
                } else {
                    if pieces[3][1] & bitboards::SQUARES[63] == 0 {
                        new_ck = false;
                    }
                }
            }
            let mut new_cq = cq;
            if cq {
                if color_to_move == 0 {
                    if pieces[3][0] & bitboards::SQUARES[0] == 0 {
                        new_cq = false;
                    }
                } else {
                    if pieces[3][1] & bitboards::SQUARES[56] == 0 {
                        new_cq = false;
                    }
                }
            }
            (new_ck, new_cq)
        }
        _ => (ck, cq)
    }
}

pub fn make_quiet_move(g: &game_state::GameState, mv: &game_state::GameMove) -> game_state::GameState {
    let color_to_move = 1 - g.color_to_move;
    let mut pieces = g.pieces.clone();
    //Make the move
    move_piece(&mut pieces, &mv, g.color_to_move);

    //Check new castle rights
    //The enemies castle right's can't change on a quiet move
    let (castle_white_kingside, castle_white_queenside) = if g.color_to_move == 0 {
        check_castle_flags(g.castle_white_kingside, g.castle_white_queenside, &mv, g.color_to_move, pieces)
    } else { (g.castle_white_kingside, g.castle_white_queenside) };
    let (castle_black_kingside, castle_black_queenside) = if g.color_to_move == 0 {
        (g.castle_black_kingside, g.castle_black_queenside)
    } else { check_castle_flags(g.castle_black_kingside, g.castle_black_queenside, &mv, g.color_to_move, pieces) };

    //This will be taken care of later
    let mut en_passant = 0u64;

    let mut half_moves = g.half_moves + 1;

    //Reset half moves if its a pawn move and also check if its a double pawn move, if so, set en passant flag
    match mv.piece_type {
        PieceType::Pawn => {
            half_moves = 0;
            if g.color_to_move == 0 && mv.to - mv.from == 16 {
                en_passant = bitboards::SQUARES[mv.to - 8];
            } else if g.color_to_move == 1 && mv.from - mv.to == 16 {
                en_passant = bitboards::SQUARES[mv.to + 8];
            }
        }
        _ => {}
    };

    //If black was to move, increase full moves by one
    let full_moves = g.full_moves + g.color_to_move;
    //Create new game state object
    game_state::GameState {
        color_to_move,
        pieces,
        castle_white_kingside,
        castle_white_queenside,
        castle_black_kingside,
        castle_black_queenside,
        en_passant,
        half_moves,
        full_moves,
    }
}

pub fn make_capture_move(g: &game_state::GameState, mv: &game_state::GameMove) -> game_state::GameState {
    let color_to_move = 1 - g.color_to_move;
    let mut pieces = g.pieces.clone();
    //Make the move
    move_piece(&mut pieces, &mv, g.color_to_move);
    //Delete to from enemy pieces
    delete_piece(&mut pieces, mv.to, color_to_move);

    let (mut castle_white_kingside, mut castle_white_queenside) = if g.color_to_move == 0 {
        check_castle_flags(g.castle_white_kingside, g.castle_white_queenside, &mv, g.color_to_move, pieces)
    } else { (g.castle_white_kingside, g.castle_white_queenside) };
    let (mut castle_black_kingside, mut castle_black_queenside) = if g.color_to_move == 0 {
        (g.castle_black_kingside, g.castle_black_queenside)
    } else { check_castle_flags(g.castle_black_kingside, g.castle_black_queenside, &mv, g.color_to_move, pieces) };

    if g.color_to_move == 0 {
        //Check that blacks rook didn't get captured
        if pieces[3][1] & bitboards::SQUARES[56] == 0 {
            castle_black_queenside = false;
        }
        if pieces[3][1] & bitboards::SQUARES[63] == 0 {
            castle_black_kingside = false;
        }
    } else {
        if pieces[3][0] & bitboards::SQUARES[0] == 0 {
            castle_white_queenside = false;
        }
        if pieces[3][0] & bitboards::SQUARES[7] == 0 {
            castle_white_kingside = false;
        }
    }
    let en_passant = 0u64;

    let half_moves = 0usize;
    let full_moves = g.full_moves + g.color_to_move;

    game_state::GameState {
        color_to_move,
        pieces,
        castle_white_kingside,
        castle_white_queenside,
        castle_black_kingside,
        castle_black_queenside,
        en_passant,
        half_moves,
        full_moves,
    }
}

pub fn make_enpassant_move(g: &game_state::GameState, mv: &game_state::GameMove) -> game_state::GameState {
    let color_to_move = 1 - g.color_to_move;
    let mut pieces = g.pieces.clone();
    //Make the move
    move_piece(&mut pieces, &mv, g.color_to_move);
    //Delete enemy pawn
    delete_piece(&mut pieces, if g.color_to_move == 0 { mv.to - 8 } else { mv.to + 8 }, color_to_move);

    let castle_white_kingside = g.castle_white_kingside;
    let castle_white_queenside = g.castle_white_queenside;
    let castle_black_kingside = g.castle_black_kingside;
    let castle_black_queenside = g.castle_black_queenside;

    let en_passant = 0u64;

    let half_moves = 0usize;
    let full_moves = g.full_moves + g.color_to_move;

    game_state::GameState {
        color_to_move,
        pieces,
        castle_white_kingside,
        castle_white_queenside,
        castle_black_kingside,
        castle_black_queenside,
        en_passant,
        half_moves,
        full_moves,
    }
}

pub fn make_castle_move(g: &game_state::GameState, mv: &game_state::GameMove) -> game_state::GameState {
    let color_to_move = 1 - g.color_to_move;
    let mut pieces = g.pieces.clone();
    //Move the king
    move_piece(&mut pieces, &mv, g.color_to_move);

    //Move the rook
    //Determine if its kingside or queenside castle
    //Kingside
    if mv.to == 58 {
        pieces[3][1] ^= bitboards::SQUARES[56];
        pieces[3][1] |= bitboards::SQUARES[59];
    } else if mv.to == 2 {
        pieces[3][0] ^= bitboards::SQUARES[0];
        pieces[3][0] |= bitboards::SQUARES[3];
    } else if mv.to == 62 {//Queenside
        pieces[3][1] ^= bitboards::SQUARES[63];
        pieces[3][1] |= bitboards::SQUARES[61];
    } else if mv.to == 6 {
        pieces[3][0] ^= bitboards::SQUARES[7];
        pieces[3][0] |= bitboards::SQUARES[5];
    }

    let (castle_white_kingside, castle_white_queenside) = if g.color_to_move == 0 { (false, false) } else {
        (g.castle_white_kingside, g.castle_white_queenside)
    };
    let (castle_black_kingside, castle_black_queenside) = if g.color_to_move == 1 { (false, false) } else {
        (g.castle_black_kingside, g.castle_black_queenside)
    };

    let en_passant = 0u64;

    let half_moves = g.half_moves + 1;
    let full_moves = g.full_moves + g.color_to_move;

    game_state::GameState {
        color_to_move,
        pieces,
        castle_white_kingside,
        castle_white_queenside,
        castle_black_kingside,
        castle_black_queenside,
        en_passant,
        half_moves,
        full_moves,
    }
}

pub fn make_promotion_move(g: &game_state::GameState, mv: &game_state::GameMove) -> game_state::GameState {
    let color_to_move = 1 - g.color_to_move;
    let mut pieces = g.pieces.clone();
    //Make the move
    move_piece(&mut pieces, &mv, g.color_to_move);
    //Delete enemy piece if any on there
    delete_piece(&mut pieces, mv.to, color_to_move);
    //Delete my pawn
    pieces[0][g.color_to_move] ^= bitboards::SQUARES[mv.to];
    //Add piece respectivly
    pieces[match mv.move_type {
        GameMoveType::Promotion(PieceType::Queen) => { 4 }
        GameMoveType::Promotion(PieceType::Knight) => { 1 }
        GameMoveType::Promotion(PieceType::Bishop) => { 2 }
        GameMoveType::Promotion(PieceType::Rook) => { 3 }
        _ => panic!("Invalid Type")
    }][g.color_to_move] |= bitboards::SQUARES[mv.to];

    let mut castle_white_kingside = g.castle_white_kingside;
    let mut castle_white_queenside = g.castle_white_queenside;
    let mut castle_black_kingside = g.castle_black_kingside;
    let mut castle_black_queenside = g.castle_black_queenside;

    if g.color_to_move == 0 {
        //Check that blacks rook didn't get captured
        if pieces[3][1] & bitboards::SQUARES[56] == 0 {
            castle_black_queenside = false;
        }
        if pieces[3][1] & bitboards::SQUARES[63] == 0 {
            castle_black_kingside = false;
        }
    } else {
        if pieces[3][0] & bitboards::SQUARES[0] == 0 {
            castle_white_queenside = false;
        }
        if pieces[3][0] & bitboards::SQUARES[7] == 0 {
            castle_white_kingside = false;
        }
    }

    let en_passant = 0u64;

    let half_moves = 0usize;
    let full_moves = g.full_moves + g.color_to_move;

    game_state::GameState {
        color_to_move,
        pieces,
        castle_white_kingside,
        castle_white_queenside,
        castle_black_kingside,
        castle_black_queenside,
        en_passant,
        half_moves,
        full_moves,
    }
}

pub fn generate_moves(g: &game_state::GameState) -> (Vec<GameMove>, bool) {
    //Following this guide:
    // https://peterellisjones.com/posts/generating-legal-chess-moves-efficiently/

    let mut move_list: Vec<GameMove> = Vec::with_capacity(60);
    let color_to_move = g.color_to_move;
    let enemy_color = 1 - color_to_move;

    //Get my pieces
    let my_king = g.pieces[5][color_to_move];
    let my_king_idx = my_king.trailing_zeros() as usize;

    let mut my_pawns = g.pieces[0][color_to_move];

    let mut my_knights = g.pieces[1][color_to_move];

    let mut my_bishops = g.pieces[2][color_to_move] | g.pieces[4][color_to_move];

    let mut my_rooks = g.pieces[3][color_to_move] | g.pieces[4][color_to_move];
    //Need this to xor out queens later, when we look at pinned pieces
    let my_queens = g.pieces[4][color_to_move];

    //Get enemy pieces
    let enemy_king = g.pieces[5][enemy_color];
    let enemy_king_idx = enemy_king.trailing_zeros() as usize;

    let enemy_pawns = g.pieces[0][enemy_color];

    let enemy_knights = g.pieces[1][enemy_color];

    let enemy_bishops = g.pieces[2][enemy_color] | g.pieces[4][enemy_color];

    let enemy_rooks = g.pieces[3][enemy_color] | g.pieces[4][enemy_color];

    let my_pieces = my_king | my_pawns | my_knights | my_bishops | my_rooks;
    let not_my_pieces = !my_pieces;
    let enemy_sliders = enemy_bishops | enemy_rooks;
    let enemy_pieces = enemy_pawns | enemy_knights | enemy_sliders | enemy_king;
    let not_enemy_pieces = !enemy_pieces;
    let all_pieces_without_my_king = enemy_pieces | (my_pieces & !my_king);
    let all_pieces = all_pieces_without_my_king | my_king;
    let empty = !all_pieces;

    let unsafe_white_squares = if color_to_move == 0 {
        get_b_attacked_squares(enemy_king_idx, enemy_pawns, enemy_knights, enemy_bishops, enemy_rooks
                               , all_pieces_without_my_king)
    } else {
        get_w_attacked_squares(enemy_king_idx, enemy_pawns, enemy_knights, enemy_bishops, enemy_rooks
                               , all_pieces_without_my_king)
    };
    let possible_king_moves = king_attack(my_king_idx) & !unsafe_white_squares & not_my_pieces;
    add_moves(&mut move_list, my_king_idx, possible_king_moves & not_enemy_pieces, &PieceType::King, GameMoveType::Quiet);
    add_moves(&mut move_list, my_king_idx, possible_king_moves & enemy_pieces, &PieceType::King, GameMoveType::Capture);
    let (king_attackers_board, checking_piece_is_slider, checking_piece_slider_is_bishop) = if unsafe_white_squares & my_king == 0 {
        (0u64, false, false)
    } else {
        if color_to_move == 0 {
            attackers_from_black(my_king, my_king_idx, enemy_pawns, enemy_knights
                                 , enemy_bishops, enemy_rooks, all_pieces)
        } else {
            attackers_from_white(my_king, my_king_idx, enemy_pawns, enemy_knights
                                 , enemy_bishops, enemy_rooks, all_pieces)
        }
    };

    let num_checkers = king_attackers_board.count_ones();
    //Double check
    if num_checkers > 1 { //Then only king moves are possible anyway
        return (move_list, true);
    }
    //Calculate capture and push mask
    let mut capture_mask = 0xFFFFFFFFFFFFFFFFu64;
    let mut push_mask = 0xFFFFFFFFFFFFFFFFu64;
    //Single-Check
    {
        if num_checkers == 1 {
            capture_mask = king_attackers_board;
            if checking_piece_is_slider {
                let checking_piece_square = king_attackers_board.trailing_zeros() as usize;
                if checking_piece_slider_is_bishop {
                    push_mask = get_bishop_ray(bishop_attack(checking_piece_square, 0u64), my_king_idx, checking_piece_square);
                } else {
                    push_mask = get_rook_ray(rook_attack(checking_piece_square, 0u64), my_king_idx, checking_piece_square);
                }
            } else {
                push_mask = 0u64;
            }
        }
    }

    //Pinned pieces
    {//Pinned by rook
        let rook_attacks_from_my_king_postion = rook_attack(my_king_idx, all_pieces);
        //See one layer through my pieces
        //If a rook is found seeing through one piece, the piece is pinned
        let xray_rooks = xray_rook_attacks(rook_attacks_from_my_king_postion, all_pieces, my_pieces, my_king_idx);
        //Go through all directions
        //Find the rooks with xray
        let mut rooks_on_xray = xray_rooks & enemy_rooks;
        while rooks_on_xray != 0 {
            let rook_position = rooks_on_xray.trailing_zeros() as usize;
            rooks_on_xray &= rooks_on_xray - 1;
            let ray = get_rook_ray(rook_attacks_from_my_king_postion | xray_rooks, rook_position, my_king_idx);
            let pinned_piece = ray & my_pieces;
            let pinned_piece_position = pinned_piece.trailing_zeros() as usize;
            if pinned_piece & my_rooks != 0 {
                let mut piece_type = PieceType::Rook;
                if pinned_piece & my_queens != 0 {
                    my_bishops ^= pinned_piece;
                    piece_type = PieceType::Queen;
                }
                my_rooks ^= pinned_piece;
                add_moves(&mut move_list, pinned_piece_position, ray & !pinned_piece & push_mask, &piece_type, GameMoveType::Quiet);
                //Don't forget that we can capture the rook aswell
                add_moves(&mut move_list, pinned_piece_position, bitboards::SQUARES[rook_position] & capture_mask, &piece_type, GameMoveType::Capture);
                continue;
            }
            if pinned_piece & my_pawns != 0 {
                my_pawns ^= pinned_piece;
                let pawn_push_once = if color_to_move == 0 { w_single_push_pawn_targets(pinned_piece, empty) } else { b_single_push_pawn_targets(pinned_piece, empty) } & ray;
                let pawn_push_twice = if color_to_move == 0 { w_double_push_pawn_targets(pinned_piece, empty) } else { b_double_push_pawn_targets(pinned_piece, empty) } & ray;
                add_moves(&mut move_list, pinned_piece_position, (pawn_push_once | pawn_push_twice) & push_mask, &PieceType::Pawn, GameMoveType::Quiet);
                continue;
            }
            if pinned_piece & my_knights != 0 {
                my_knights ^= pinned_piece;
                continue;
            }
            if pinned_piece & my_bishops != 0 {
                my_bishops ^= pinned_piece;
            }
        }
        //Pinned by bishop
        let bishop_attacks_from_my_king_position = bishop_attack(my_king_idx, all_pieces);
        let xray_bishops = xray_bishop_attacks(bishop_attacks_from_my_king_position, all_pieces, my_pieces, my_king_idx);
        let mut bishops_on_xray = xray_bishops & enemy_bishops;
        while bishops_on_xray != 0 {
            let bishop_position = bishops_on_xray.trailing_zeros() as usize;
            bishops_on_xray &= bishops_on_xray - 1;
            let ray = get_bishop_ray(bishop_attacks_from_my_king_position | xray_bishops, bishop_position, my_king_idx);
            let pinned_piece = ray & my_pieces;
            let pinned_piece_position = pinned_piece.trailing_zeros() as usize;
            if pinned_piece & my_bishops != 0 {
                let mut piece_type = PieceType::Bishop;
                if pinned_piece & my_queens != 0 {
                    my_rooks ^= pinned_piece;
                    piece_type = PieceType::Queen;
                }
                my_bishops ^= pinned_piece;

                add_moves(&mut move_list, pinned_piece_position, ray & !pinned_piece & push_mask, &piece_type, GameMoveType::Quiet);
                add_moves(&mut move_list, pinned_piece_position, bitboards::SQUARES[bishop_position] & capture_mask, &piece_type, GameMoveType::Capture);
                continue;
            }
            if pinned_piece & my_pawns != 0 {
                my_pawns ^= pinned_piece;
                let pawn_targets = if color_to_move == 0 { w_pawn_east_targets(pinned_piece) } else { b_pawn_east_targets(pinned_piece) } | if color_to_move == 0 { w_pawn_west_targets(pinned_piece) } else { b_pawn_west_targets(pinned_piece) };
                let pawn_captures = pawn_targets & bitboards::SQUARES[bishop_position] & capture_mask;
                //let pawn_enpassants = pawn_targets & g.en_passant;
                add_moves(&mut move_list, pinned_piece_position, pawn_captures, &PieceType::Pawn, GameMoveType::Capture);
                //add_moves(&mut move_list, pinned_piece_position, pawn_enpassants, &PieceType::Pawn, GameMoveType::EnPassant);
                continue;
            }
            if pinned_piece & my_knights != 0 {
                my_knights ^= pinned_piece;
                continue;
            }
            if pinned_piece & my_rooks != 0 {
                my_rooks ^= pinned_piece;
            }
        }
    }
    //Calculate normal moves
    //Pawns
    {
        //Single push
        {
            let my_pawns_single_push = if color_to_move == 0 { w_single_push_pawn_targets(my_pawns, empty) } else { b_single_push_pawn_targets(my_pawns, empty) } & push_mask;
            let my_pawns_no_promotion = my_pawns_single_push & !bitboards::RANKS[if color_to_move == 0 { 7 } else { 0 }];
            let my_pawns_promotion = my_pawns_single_push & bitboards::RANKS[if color_to_move == 0 { 7 } else { 0 }];
            add_quiet_pawn_single_pushes(my_pawns_no_promotion, &color_to_move, &mut move_list);
            add_promotion_push(my_pawns_promotion, &color_to_move, &mut move_list, 8usize);
        }
        //Double push
        {
            let my_pawns_double_push = if color_to_move == 0 { w_double_push_pawn_targets(my_pawns, empty) } else { b_double_push_pawn_targets(my_pawns, empty) } & push_mask;
            add_quiet_pawn_double_pushes(my_pawns_double_push, &color_to_move, &mut move_list);
        }
        //Capture west
        {
            let my_pawns_west_targets = if color_to_move == 0 { w_pawn_west_targets(my_pawns) } else { b_pawn_west_targets(my_pawns) };
            let my_pawns_west_normal_captures = my_pawns_west_targets & capture_mask & enemy_pieces;
            //Checking for promotion on capture
            let my_pawns_no_promotion = my_pawns_west_normal_captures & !bitboards::RANKS[if color_to_move == 0 { 7 } else { 0 }];
            let my_pawns_promotion = my_pawns_west_normal_captures & bitboards::RANKS[if color_to_move == 0 { 7 } else { 0 }];
            //Capture
            add_pawn_capture(my_pawns_no_promotion, &color_to_move, &mut move_list, 7usize);
            //Promotion capture
            add_promotion_push(my_pawns_promotion, &color_to_move, &mut move_list, 7usize);


            //En passant
            //We can capture en passant, if its in capture mask aswell
            let my_pawns_west_enpassants = my_pawns_west_targets & g.en_passant & if color_to_move == 0 { capture_mask << 8 } else { capture_mask >> 8 };
            add_en_passants(my_pawns_west_enpassants, &color_to_move, &mut move_list, 7usize, all_pieces_without_my_king, enemy_rooks, my_king_idx);
        }
        //Capture east
        {
            let my_pawns_east_targets = if color_to_move == 0 { w_pawn_east_targets(my_pawns) } else { b_pawn_east_targets(my_pawns) };
            let my_pawns_east_normal_captures = my_pawns_east_targets & capture_mask & enemy_pieces;
            //Checking for promotion on capture
            let my_pawns_no_promotion = my_pawns_east_normal_captures & !bitboards::RANKS[if color_to_move == 0 { 7 } else { 0 }];
            let my_pawns_promotion = my_pawns_east_normal_captures & bitboards::RANKS[if color_to_move == 0 { 7 } else { 0 }];

            add_pawn_capture(my_pawns_no_promotion, &color_to_move, &mut move_list, 9usize);
            add_promotion_push(my_pawns_promotion, &color_to_move, &mut move_list, 9usize);
            //En passants
            let  my_pawns_east_enpassants = my_pawns_east_targets & g.en_passant & if color_to_move == 0 { capture_mask << 8 } else { capture_mask >> 8 };
            add_en_passants(my_pawns_east_enpassants, &color_to_move, &mut move_list, 9usize, all_pieces_without_my_king, enemy_rooks, my_king_idx);
        }
    }
    //Knights
    {
        while my_knights != 0u64 {
            let index = if color_to_move == 0 { 63usize - my_knights.leading_zeros() as usize } else { my_knights.trailing_zeros() as usize };
            let my_knight_attacks = knight_attack(index) & not_my_pieces;
            let my_knight_captures = my_knight_attacks & enemy_pieces & capture_mask;
            add_moves(&mut move_list, index, my_knight_captures, &PieceType::Knight, GameMoveType::Capture);
            let my_knight_quiets = my_knight_attacks & !enemy_pieces & push_mask;
            add_moves(&mut move_list, index, my_knight_quiets, &PieceType::Knight, GameMoveType::Quiet);
            my_knights ^= 1u64 << index;
        }
    }
    //Bishops
    {
        while my_bishops != 0u64 {
            let index = if color_to_move == 0 { 63usize - my_bishops.leading_zeros() as usize } else { my_bishops.trailing_zeros() as usize };
            let piece= 1u64<<index;
            let my_bishop_attack = bishop_attack(index, all_pieces) & not_my_pieces;
            let my_bishop_capture = my_bishop_attack & enemy_pieces & capture_mask;
            let piece_type = if piece & my_queens != 0 { PieceType::Queen } else { PieceType::Bishop };
            add_moves(&mut move_list, index, my_bishop_capture, &piece_type, GameMoveType::Capture);
            let my_bishop_quiet = my_bishop_attack & !enemy_pieces & push_mask;
            add_moves(&mut move_list, index, my_bishop_quiet, &piece_type, GameMoveType::Quiet);
            my_bishops ^= piece;
        }
    }
    //Rooks
    {
        while my_rooks != 0u64 {
            let index = if color_to_move == 0 { 63usize - my_rooks.leading_zeros() as usize } else { my_rooks.trailing_zeros() as usize };
            let piece= 1u64<<index;
            let my_rook_attack = rook_attack(index, all_pieces) & not_my_pieces;
            let my_rook_capture = my_rook_attack & enemy_pieces & capture_mask;
            let piece_type = if piece& my_queens != 0 { PieceType::Queen } else { PieceType::Rook };
            add_moves(&mut move_list, index, my_rook_capture, &piece_type, GameMoveType::Capture);
            let my_rook_quiets = my_rook_attack & !enemy_pieces & push_mask;
            add_moves(&mut move_list, index, my_rook_quiets, &piece_type, GameMoveType::Quiet);
            my_rooks ^= piece;
        }
    }
    //Castles
    if num_checkers == 0 {
        if g.color_to_move == 0 {
            //Make sure there is no piece in between and safe squares
            if g.castle_white_kingside {
                if (all_pieces | unsafe_white_squares) & (bitboards::SQUARES[5] | bitboards::SQUARES[6]) == 0 {
                    move_list.push(GameMove {
                        from: my_king_idx,
                        to: 6usize,
                        move_type: GameMoveType::Castle,
                        piece_type: PieceType::King,
                    });
                }
            }
            if g.castle_white_queenside {
                if ((all_pieces | unsafe_white_squares) & (bitboards::SQUARES[2] | bitboards::SQUARES[3]) | all_pieces & bitboards::SQUARES[1]) == 0 {
                    move_list.push(GameMove {
                        from: my_king_idx,
                        to: 2usize,
                        move_type: GameMoveType::Castle,
                        piece_type: PieceType::King,
                    });
                }
            }
        } else {
            if g.castle_black_kingside {
                if (all_pieces | unsafe_white_squares) & (bitboards::SQUARES[61] | bitboards::SQUARES[62]) == 0 {
                    move_list.push(GameMove {
                        from: my_king_idx,
                        to: 62usize,
                        move_type: GameMoveType::Castle,
                        piece_type: PieceType::King,
                    });
                }
            }
            if g.castle_black_queenside {
                if ((all_pieces | unsafe_white_squares) & (bitboards::SQUARES[58] | bitboards::SQUARES[59]) | all_pieces & bitboards::SQUARES[57]) == 0 {
                    move_list.push(GameMove {
                        from: my_king_idx,
                        to: 58usize,
                        move_type: GameMoveType::Castle,
                        piece_type: PieceType::King,
                    });
                }
            }
        }
    }
    (move_list, num_checkers > 0)
}

#[inline(always)]
pub fn add_quiet_pawn_single_pushes(mut single_push_board: u64, color_to_move: &usize, move_list: &mut Vec<GameMove>) {
    while single_push_board != 0u64 {
        let idx = single_push_board.trailing_zeros() as usize;
        move_list.push(GameMove {
            from: if *color_to_move == 0 { idx - 8 } else { idx + 8 },
            to: idx,
            move_type: GameMoveType::Quiet,
            piece_type: PieceType::Pawn,
        });
        single_push_board ^= 1 << idx;
    }
}

#[inline(always)]
pub fn add_quiet_pawn_double_pushes(mut double_push_board: u64, color_to_move: &usize, move_list: &mut Vec<GameMove>) {
    while double_push_board != 0u64 {
        let idx = double_push_board.trailing_zeros() as usize;
        move_list.push(GameMove {
            from: if *color_to_move == 0 { idx - 16 } else { idx + 16 },
            to: idx,
            move_type: GameMoveType::Quiet,
            piece_type: PieceType::Pawn,
        });
        double_push_board ^= 1 << idx;
    }
}

#[inline(always)]
pub fn add_promotion_push(mut promotion_board: u64, color_to_move: &usize, move_list: &mut Vec<GameMove>, source_shift: usize) {
    while promotion_board != 0u64 {
        let idx = promotion_board.trailing_zeros() as usize;
        move_list.push(GameMove {
            from: if *color_to_move == 0 { idx - source_shift } else { idx + source_shift },
            to: idx,
            move_type: GameMoveType::Promotion(PieceType::Queen),
            piece_type: PieceType::Pawn,
        });
        move_list.push(GameMove {
            from: if *color_to_move == 0 { idx - source_shift } else { idx + source_shift },
            to: idx,
            move_type: GameMoveType::Promotion(PieceType::Rook),
            piece_type: PieceType::Pawn,
        });
        move_list.push(GameMove {
            from: if *color_to_move == 0 { idx - source_shift } else { idx + source_shift },
            to: idx,
            move_type: GameMoveType::Promotion(PieceType::Bishop),
            piece_type: PieceType::Pawn,
        });
        move_list.push(GameMove {
            from: if *color_to_move == 0 { idx - source_shift } else { idx + source_shift },
            to: idx,
            move_type: GameMoveType::Promotion(PieceType::Knight),
            piece_type: PieceType::Pawn,
        });
        promotion_board ^= 1 << idx;
    }
}

#[inline(always)]
pub fn add_pawn_capture(mut capture_board: u64, color_to_move: &usize, move_list: &mut Vec<GameMove>, source_shift: usize) {
    while capture_board != 0u64 {
        let idx = capture_board.trailing_zeros() as usize;
        move_list.push(GameMove {
            from: if *color_to_move == 0 { idx - source_shift } else { idx + source_shift },
            to: idx,
            move_type: GameMoveType::Capture,
            piece_type: PieceType::Pawn,
        });
        capture_board ^= 1 << idx;
    }
}

#[inline(always)]
pub fn add_en_passants(mut enpassant_board: u64, color_to_move: &usize, move_list: &mut Vec<GameMove>, source_shift: usize, all_pieces_without_my_king: u64, enemy_rooks: u64, my_king_idx: usize) {
    while enpassant_board != 0u64 {
        let index = enpassant_board.trailing_zeros() as usize;
        enpassant_board ^= 1u64 << index;
        //Check if rare case didn't happen
        //Remove t-7,t-8 or t+7,t+8
        let all_pieces_without_en_passants = all_pieces_without_my_king & !bitboards::SQUARES[if *color_to_move == 0 { index - source_shift } else { index + source_shift }]
            & !bitboards::SQUARES[if *color_to_move == 0 { index - 8 } else { index + 8 }];
        if rook_attack(my_king_idx, all_pieces_without_en_passants) & (!bitboards::FILES[my_king_idx % 8]) & enemy_rooks != 0 {
            continue;
        }
        move_list.push(GameMove {
            from: if *color_to_move == 0 { index - source_shift } else { index + source_shift },
            to: index,
            move_type: GameMoveType::EnPassant,
            piece_type: PieceType::Pawn,
        });
    }
}

pub fn xray_rook_attacks(attacks: u64, occupied_square: u64, one_time_blockers: u64, rook_square: usize) -> u64 {
    return attacks ^ rook_attack(rook_square, occupied_square ^ (one_time_blockers & attacks));
}

pub fn xray_bishop_attacks(attacks: u64, occupied_square: u64, one_time_blockers: u64, bishop_square: usize) -> u64 {
    return attacks ^ bishop_attack(bishop_square, occupied_square ^ (one_time_blockers & attacks));
}

//Gets the ray of one rook into a specific direction
pub fn get_rook_ray(rook_attacks_in_all_directions: u64, target_square: usize, rook_square: usize) -> u64 {
    let diff = target_square as isize - rook_square as isize;
    let target_rank = target_square / 8;
    let target_file = target_square % 8;
    let rook_rank = rook_square / 8;
    let rook_file = rook_square % 8;
    if diff > 0 {
        //Same vertical
        if target_rank == rook_rank {
            return bitboards::FILES_LESS_THAN[target_file] & bitboards::FILES_GREATER_THAN[rook_file] & rook_attacks_in_all_directions;
        } else {
            return bitboards::RANKS_LESS_THAN[target_rank] & bitboards::RANKS_GREATER_THAN[rook_rank] & rook_attacks_in_all_directions;
        }
    } else {
        if target_rank == rook_rank {
            return bitboards::FILES_GREATER_THAN[target_file] & bitboards::FILES_LESS_THAN[rook_file] & rook_attacks_in_all_directions;
        } else {
            return bitboards::RANKS_GREATER_THAN[target_rank] & bitboards::RANKS_LESS_THAN[rook_rank] & rook_attacks_in_all_directions;
        }
    }
}

//Gets the rof of one bishop into a specific direction
pub fn get_bishop_ray(bishop_attack_in_all_directions: u64, target_square: usize, bishop_square: usize) -> u64 {
    let diff = target_square as isize - bishop_square as isize;
    let target_rank = target_square / 8;
    let target_file = target_square % 8;
    let bishop_rank = bishop_square / 8;
    let bishop_file = bishop_square % 8;
    if diff > 0 {
        if diff % 7 == 0 {
            return bitboards::FILES_GREATER_THAN[target_file] & bitboards::FILES_LESS_THAN[bishop_file]
                & bitboards::RANKS_LESS_THAN[target_rank] & bitboards::RANKS_GREATER_THAN[bishop_rank]
                & bishop_attack_in_all_directions;
        } else {
            return bitboards::FILES_LESS_THAN[target_file] & bitboards::FILES_GREATER_THAN[bishop_file]
                & bitboards::RANKS_LESS_THAN[target_rank] & bitboards::RANKS_GREATER_THAN[bishop_rank]
                & bishop_attack_in_all_directions;
        }
    } else {
        if diff % -7 == 0 {
            return bitboards::FILES_LESS_THAN[target_file] & bitboards::FILES_GREATER_THAN[bishop_file]
                & bitboards::RANKS_GREATER_THAN[target_rank] & bitboards::RANKS_LESS_THAN[bishop_rank]
                & bishop_attack_in_all_directions;
        } else {
            return bitboards::FILES_GREATER_THAN[target_file] & bitboards::FILES_LESS_THAN[bishop_file]
                & bitboards::RANKS_GREATER_THAN[target_rank] & bitboards::RANKS_LESS_THAN[bishop_rank]
                & bishop_attack_in_all_directions;
        }
    }
}

pub fn attackers_from_white(square_board: u64, square: usize, white_pawns: u64, white_knights: u64, white_bishops: u64, white_rooks: u64, blockers: u64) -> (u64, bool, bool) {
    let mut attackers = 0u64;
    let mut slider_flag = false;
    let mut bishop_slider = false;
    attackers |= knight_attack(square) & white_knights;
    attackers |= (b_pawn_west_targets(square_board) | b_pawn_east_targets(square_board)) & white_pawns;
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

pub fn attackers_from_black(square_board: u64, square: usize, black_pawns: u64, black_knights: u64, black_bishops: u64, black_rooks: u64, blockers: u64) -> (u64, bool, bool) {
    let mut attackers = 0u64;
    let mut slider_flag = false;
    let mut bishop_slider = false;
    attackers |= knight_attack(square) & black_knights;
    attackers |= (w_pawn_west_targets(square_board) | w_pawn_east_targets(square_board)) & black_pawns;
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

pub fn get_w_attacked_squares(white_king_idx: usize, white_pawns: u64, mut white_knights: u64, mut white_bishops: u64, mut white_rooks: u64, blocked_squares: u64) -> u64 {
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

pub fn get_b_attacked_squares(black_king_idx: usize, black_pawns: u64, mut black_knights: u64, mut black_bishops: u64, mut black_rooks: u64, blocked_squares: u64) -> u64 {
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
