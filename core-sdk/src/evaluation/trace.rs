use crate::board_representation::game_state::{BLACK, WHITE};
use crate::evaluation::parameters::{normal_parameters::*, special_parameters::*, *};

pub struct TraceEntry(pub u16, pub i8);
pub struct CollapsedTrace {
    pub phase: f32,
    pub entries: Vec<TraceEntry>,
    pub pawns_on_board: u8,
    pub knights: i8,
    pub attackers: [u8; 2],
    pub knight_attacked_sq: [u8; 2],
    pub bishop_attacked_sq: [u8; 2],
    pub rook_attacked_sq: [u8; 2],
    pub queen_attacked_sq: [u8; 2],
    pub knight_safe_check: [u8; 2],
    pub bishop_safe_check: [u8; 2],
    pub rook_safe_check: [u8; 2],
    pub queen_safe_check: [u8; 2],
    pub is_guaranteed_draw: bool,
    pub slightly_winning_no_pawn: bool,
    pub slightly_winning_enemy_can_sac: bool,
}
impl CollapsedTrace {
    pub fn evaluate(&self, params: &Parameters) -> f32 {
        if self.is_guaranteed_draw {
            return 0.;
        }
        let mut res = (0., 0.);
        for entry in self.entries.iter() {
            res.0 += params.normal[0][entry.0 as usize] * f32::from(entry.1);
            res.1 += params.normal[1][entry.0 as usize] * f32::from(entry.1);
        }

        res.0 += (params.special[IDX_ATTACK_WEIGHT + 2 * self.attackers[WHITE] as usize]
            * params.special[IDX_SAFETY_TABLE
                + 2 * ((f32::from(self.knight_attacked_sq[WHITE])
                    * params.special[IDX_KNIGHT_ATTACK_VALUE]
                    + f32::from(self.bishop_attacked_sq[WHITE])
                        * params.special[IDX_BISHOP_ATTACK_VALUE]
                    + f32::from(self.rook_attacked_sq[WHITE])
                        * params.special[IDX_ROOK_ATTACK_VALUE]
                    + f32::from(self.queen_attacked_sq[WHITE])
                        * params.special[IDX_QUEEN_ATTACK_VALUE]
                    + f32::from(self.knight_safe_check[WHITE])
                        * params.special[IDX_KNIGHT_CHECK_VALUE]
                    + f32::from(self.bishop_safe_check[WHITE])
                        * params.special[IDX_BISHOP_CHECK_VALUE]
                    + f32::from(self.rook_safe_check[WHITE]) * params.special[IDX_ROOK_CHECK_VALUE]
                    + f32::from(self.queen_safe_check[WHITE])
                        * params.special[IDX_QUEEN_CHECK_VALUE]) as usize)
                    .max(0)
                    .min(99)]
            - params.special[IDX_ATTACK_WEIGHT + 2 * self.attackers[BLACK] as usize]
                * params.special[IDX_SAFETY_TABLE
                    + 2 * ((f32::from(self.knight_attacked_sq[BLACK])
                        * params.special[IDX_KNIGHT_ATTACK_VALUE]
                        + f32::from(self.bishop_attacked_sq[BLACK])
                            * params.special[IDX_BISHOP_ATTACK_VALUE]
                        + f32::from(self.rook_attacked_sq[BLACK])
                            * params.special[IDX_ROOK_ATTACK_VALUE]
                        + f32::from(self.queen_attacked_sq[BLACK])
                            * params.special[IDX_QUEEN_ATTACK_VALUE]
                        + f32::from(self.knight_safe_check[BLACK])
                            * params.special[IDX_KNIGHT_CHECK_VALUE]
                        + f32::from(self.bishop_safe_check[BLACK])
                            * params.special[IDX_BISHOP_CHECK_VALUE]
                        + f32::from(self.rook_safe_check[BLACK])
                            * params.special[IDX_ROOK_CHECK_VALUE]
                        + f32::from(self.queen_safe_check[BLACK])
                            * params.special[IDX_QUEEN_CHECK_VALUE])
                        as usize)
                        .max(0)
                        .min(99)])
            / 100.0;
        res.1 += (params.special[IDX_ATTACK_WEIGHT + 2 * self.attackers[WHITE] as usize + 1]
            * params.special[IDX_SAFETY_TABLE
                + 2 * ((f32::from(self.knight_attacked_sq[WHITE])
                    * params.special[IDX_KNIGHT_ATTACK_VALUE + 1]
                    + f32::from(self.bishop_attacked_sq[WHITE])
                        * params.special[IDX_BISHOP_ATTACK_VALUE + 1]
                    + f32::from(self.rook_attacked_sq[WHITE])
                        * params.special[IDX_ROOK_ATTACK_VALUE + 1]
                    + f32::from(self.queen_attacked_sq[WHITE])
                        * params.special[IDX_QUEEN_ATTACK_VALUE + 1]
                    + f32::from(self.knight_safe_check[WHITE])
                        * params.special[IDX_KNIGHT_CHECK_VALUE + 1]
                    + f32::from(self.bishop_safe_check[WHITE])
                        * params.special[IDX_BISHOP_CHECK_VALUE + 1]
                    + f32::from(self.rook_safe_check[WHITE])
                        * params.special[IDX_ROOK_CHECK_VALUE + 1]
                    + f32::from(self.queen_safe_check[WHITE])
                        * params.special[IDX_QUEEN_CHECK_VALUE + 1])
                    as usize)
                    .max(0)
                    .min(99)
                + 1]
            - params.special[IDX_ATTACK_WEIGHT + 2 * self.attackers[BLACK] as usize + 1]
                * params.special[IDX_SAFETY_TABLE
                    + 2 * ((f32::from(self.knight_attacked_sq[BLACK])
                        * params.special[IDX_KNIGHT_ATTACK_VALUE + 1]
                        + f32::from(self.bishop_attacked_sq[BLACK])
                            * params.special[IDX_BISHOP_ATTACK_VALUE + 1]
                        + f32::from(self.rook_attacked_sq[BLACK])
                            * params.special[IDX_ROOK_ATTACK_VALUE + 1]
                        + f32::from(self.queen_attacked_sq[BLACK])
                            * params.special[IDX_QUEEN_ATTACK_VALUE + 1]
                        + f32::from(self.knight_safe_check[BLACK])
                            * params.special[IDX_KNIGHT_CHECK_VALUE + 1]
                        + f32::from(self.bishop_safe_check[BLACK])
                            * params.special[IDX_BISHOP_CHECK_VALUE + 1]
                        + f32::from(self.rook_safe_check[BLACK])
                            * params.special[IDX_ROOK_CHECK_VALUE + 1]
                        + f32::from(self.queen_safe_check[BLACK])
                            * params.special[IDX_QUEEN_CHECK_VALUE + 1])
                        as usize)
                        .max(0)
                        .min(99)
                    + 1])
            / 100.0;

        res.0 += params.special[IDX_KNIGHT_VALUE_WITH_PAWN + self.pawns_on_board as usize]
            * f32::from(self.knights);
        res.1 += params.special[IDX_KNIGHT_VALUE_WITH_PAWN + self.pawns_on_board as usize]
            * f32::from(self.knights);

        if self.slightly_winning_no_pawn {
            res = (res.0, res.1 * params.special[IDX_SLIGHTLY_WINNING_NO_PAWN]);
        } else if self.slightly_winning_enemy_can_sac {
            res = (
                res.0,
                res.1 * params.special[IDX_SLIGHTLY_WINNING_ENEMY_CAN_SAC],
            );
        }
        (res.0 * self.phase + res.1 / 1.5 * (128.0 - self.phase)) / 128.0
    }
}
pub struct LargeTrace {
    pub phase: f32,
    pub normal_coeffs: [i8; NORMAL_PARAMS],
    pub pawns_on_board: u8,
    pub knights: i8,
    pub attackers: [u8; 2],
    pub knight_attacked_sq: [u8; 2],
    pub bishop_attacked_sq: [u8; 2],
    pub rook_attacked_sq: [u8; 2],
    pub queen_attacked_sq: [u8; 2],
    pub knight_safe_check: [u8; 2],
    pub bishop_safe_check: [u8; 2],
    pub rook_safe_check: [u8; 2],
    pub queen_safe_check: [u8; 2],
    pub is_guaranteed_draw: bool,
    pub slightly_winning_no_pawn: bool,
    pub slightly_winning_enemy_can_sac: bool,
}

impl LargeTrace {
    pub fn default() -> Self {
        LargeTrace {
            phase: 0.,
            normal_coeffs: [0; NORMAL_PARAMS],
            pawns_on_board: 0,
            knights: 0,
            attackers: [0; 2],
            knight_attacked_sq: [0; 2],
            bishop_attacked_sq: [0; 2],
            rook_attacked_sq: [0; 2],
            queen_attacked_sq: [0; 2],
            knight_safe_check: [0; 2],
            bishop_safe_check: [0; 2],
            rook_safe_check: [0; 2],
            queen_safe_check: [0; 2],
            is_guaranteed_draw: false,
            slightly_winning_no_pawn: false,
            slightly_winning_enemy_can_sac: false,
        }
    }

    pub fn collapse(self) -> CollapsedTrace {
        let mut entries = Vec::new();
        for i in 0..NORMAL_PARAMS {
            if self.normal_coeffs[i] != 0 {
                entries.push(TraceEntry(i as u16, self.normal_coeffs[i]));
            }
        }
        CollapsedTrace {
            phase: self.phase,
            entries,
            knights: self.knights,
            pawns_on_board: self.pawns_on_board,
            attackers: self.attackers,
            knight_attacked_sq: self.knight_attacked_sq,
            bishop_attacked_sq: self.bishop_attacked_sq,
            rook_attacked_sq: self.rook_attacked_sq,
            queen_attacked_sq: self.queen_attacked_sq,
            knight_safe_check: self.knight_safe_check,
            bishop_safe_check: self.bishop_safe_check,
            rook_safe_check: self.rook_safe_check,
            queen_safe_check: self.queen_safe_check,
            is_guaranteed_draw: self.is_guaranteed_draw,
            slightly_winning_no_pawn: self.slightly_winning_no_pawn,
            slightly_winning_enemy_can_sac: self.slightly_winning_enemy_can_sac,
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "texel-tuning")]
    use super::super::parameters::Parameters;
    #[cfg(feature = "texel-tuning")]
    use crate::board_representation::game_state::GameState;
    #[cfg(feature = "texel-tuning")]
    use crate::evaluation::eval_game_state;

    #[test]
    #[ignore]
    pub fn traceeval() {
        if !cfg!(feature = "texel-tuning") {
            panic!("Feature texel-tuning has to be enabled");
        }
        #[cfg(feature = "texel-tuning")]
        {
            let positions: &str = "3r1r1k/pb2b3/1p1q3p/1Pnp1pp1/P7/1QN1PN2/5PPP/1R1R1BK1 w - - 0 21
8/8/p4pK1/4kPp1/8/8/P5P1/8 w - - 0 1
8/8/3k4/1p5R/4r3/1K6/8/8 b - - 0 1
8/p4p2/1p2p3/3kP1Nn/5P2/3K4/P7/8 w - - 0 1
7R/8/1k6/1p6/1K6/8/8/2r5 b - - 0 1
1k6/3n4/8/8/5P2/8/4N3/1K6 w - - 0 1
1k6/3n4/8/8/8/8/8/1K6 w - - 0 1
1k6/8/8/8/5p2/8/4N3/1K6 w - - 0 1
r1q1kr2/1bp1n1np/p7/1p3pp1/3N4/NP2R1P1/3PP1BP/R4QK1 w q - 2 21
4r1k1/1pqnnp1p/p3b1p1/P3p3/8/1NPB4/2P3PP/R3QR1K w - - 0 21
r1q1rbk1/pp1n2pp/2p1np2/5N1b/N3PP2/6PP/PPQB2B1/4RRK1 w - - 0 21
r3b1k1/pp2bpp1/2n1p2p/8/2B5/2P1BN1P/PP3PP1/3R2K1 w - - 0 21
5rk1/1r2npp1/1b1p3p/p1pP4/R3P3/5N1P/1qN2PP1/3QR1K1 w - - 0 21
r4rk1/2p1b1pp/1p1p2q1/p1nPp3/2P4P/2N2NP1/PP2QP1K/2R2R2 w - - 2 21
r1n4r/p1n2k2/1pp1bpp1/4p2p/1PP1P3/PNN3P1/5P1P/R2R1BK1 w - - 0 21
r1r3k1/1pq1bppp/3p1n2/3P4/3Q4/5P2/PRP1B1PP/2NR3K w - - 1 21
2r3k1/1p1nrppp/p1qp4/8/P1b1PR2/2PQ4/1P2B1PP/3R2BK w - - 1 21
1k1rr2b/1bp4p/pn1pp1p1/3n4/qp1PNPP1/3B2N1/PPPB1Q1P/1K1R1R2 w - - 4 21
r4rk1/2q1ppbp/1n4p1/pR1pP3/Pn1P4/4BP2/1N1QN1PP/R5K1 w - - 1 21
7r/1p1bk2p/p2pNp1b/8/2B1P3/2n5/2P3PP/3RK2R w - - 0 21
r5k1/p4ppp/1p2rn2/1B1qn3/3B4/PP3P2/3Q1KPP/3RR3 w - - 8 21
2b1r1k1/r2n1p1p/5npq/1QN1p3/7P/p1P2B2/PP1N1P2/2K1R2R w - - 0 21
2kr1b1r/pp4p1/4qn1p/8/3p3P/6N1/PPPB2P1/2KRR3 w - - 0 21
3r2k1/1pp2ppp/3n4/p1Np4/3P4/2P5/PP3PPP/4R1K1 w - - 0 21
r3r3/pb1q1k2/1p3n1p/2b5/3p1B2/P1N1P1P1/1P4BP/2RQR1K1 w - - 0 21
r5k1/3bbppp/3p1n2/np2p3/p2PP3/P2BBN1P/1P1N1PP1/2r1R1K1 w - - 0 21
r1r2k2/3bppb1/p2p2pp/3P2BP/2B3P1/1P3P2/P2K4/2R4R w - - 0 21
2r3k1/qp1r1ppp/p2p1n2/4p3/2PnP1b1/1PN1R1P1/P1NQ1P1P/2R2BK1 w - - 3 21
2k4r/2p2ppp/p2r4/Pp6/4p3/5N2/P1p2PPP/2R1R1K1 w - - 0 21
2r2rk1/1pqnbpp1/p3pn1p/8/2P1N3/1P1Q2NP/P2B1PP1/R2R2K1 w - - 10 21
r4rk1/4bppp/5n2/4p3/P3P3/2B3P1/1Pq2PBP/RN3QK1 w - - 3 21
5rk1/1pr1q1pp/pB1pp3/4p2n/4P3/5PP1/PPP2Q2/1K1R1B2 w - - 0 21
2r3k1/3p1rpp/b1q1pp2/p1b5/2P1P3/P4NQ1/1B3PPP/3RR1K1 w - - 3 21
b2rr3/1qn1ppkp/pp1p1np1/2pP4/P3P3/2N1QN1P/1PPR1PP1/3R1BK1 w - - 0 21
r2q1rk1/p1p3b1/1p6/2pPpbpp/2P5/2N2P1P/PP2QBP1/R4RK1 w - - 0 21
r1r3k1/p4ppp/1pn1pn2/8/3PN3/P1PK1P2/3B2PP/1R5R w - - 1 21
1rbq1rk1/1n3p2/p2p2pb/2pP3n/1pP1Pp2/1P1Q2Pp/PB1NN2P/1R3RKB w - - 0 21
2r3k1/1brqb2p/p4pp1/1p1pp3/3P1P2/3BP3/PPRN1KPP/2RQ4 w - - 0 21
r3r1k1/1p1b2bn/p2p1qn1/2pP1p1B/P4P1p/2N1B2N/1P1Q2PP/R4RK1 w - - 3 21
r3kn1r/1b4b1/pp2pnpp/4q3/P2N2QB/2N5/1PP3PP/3R1RK1 w kq - 2 21
2rr2k1/pp2qpp1/4p3/7p/2BnP1n1/P1N3P1/1P3PP1/2RR1QK1 w - - 2 21
2rbr2k/1p3ppp/p2q1n2/4pR2/P1n1P3/2NBB1Q1/1PP3PP/5R1K w - - 7 21
r4rk1/pb3pp1/5n1p/3qnB2/P2p4/6BQ/1P1N1PPP/R4RK1 w - - 1 21
r5k1/pp1b1pbp/4p1p1/3p4/1P1P4/2q3P1/P4P1P/R2QRBK1 w - - 0 21
2r2rk1/p3qpp1/1p2p2p/3bn3/1P6/P3PPPN/3QB1P1/2R2RK1 w - - 2 21
r1b1r1k1/p4pp1/1p5p/8/2NPpB2/2n1P3/PR3PPP/R5K1 w - - 0 21
4r1k1/pp3ppp/8/4Bb2/2Pp4/q3PB2/P4PPP/R1R3K1 w - - 0 21
2r2r2/1p1b1pkp/p2ppp2/4nP1B/4PQ2/qNN5/P1P3PP/5R1K w - - 0 21
r1b1qrk1/1pp4p/3p2nP/p2P2Bn/3Q2p1/1BP5/P1P1N1P1/R4RK1 w - - 5 21
r4rk1/p1q2pp1/4pn1p/8/1pPP1bP1/3B1N2/PP2Q2P/2KR2R1 w - - 1 21
2kr1b1r/1b4p1/p3Rn2/6B1/3pN3/3B4/PPP3pP/5R1K w - - 0 21
2r2r1k/pp1b2pp/8/3Nbpq1/1P1Pn3/P2B3P/1B3P1P/R2QRK2 w - - 0 21
4k2r/pp3p1p/2pr2p1/3NqP2/2p1P1b1/8/PPPQ1RP1/4R1K1 w k - 5 21
r1r3k1/4bpp1/pq1npnp1/1pNp2B1/1P1P4/P1NQP2P/5PP1/R1R3K1 w - - 5 21
2krr3/Qbpp3p/n4q2/1Pp1bp2/2P4p/PN6/1B3PP1/2KR1B1R w - - 1 21
r7/pb2kp2/1p2p2p/8/3r4/P3P3/1P3KPP/5BR1 w - - 0 21
r3k1r1/4bp1p/2bppp2/4qP2/1pB1PN2/p7/PPP2QPP/1K1RR3 w q - 0 21
k2r3r/pb2qpp1/2n1p2p/2pp4/R7/2PBPN2/PQ3PPP/K6R w - d6 0 21
r3r1k1/1p1nppb1/2np2p1/7p/1p1PP3/P1N1BB1P/1P3PP1/R2R2K1 w - - 0 21
r3kb2/1p3bp1/p1n1p3/2PpP3/6Pq/2N1BQ2/PPP5/R3K2B w Qq - 7 21
3q1rk1/3rbppp/ppn1p3/4B3/1PN1P3/PQ4P1/5P1P/2RR2K1 w - - 1 21
qn1r2k1/1br1bpp1/pp2pn1p/4p3/1PP4B/P1N2N2/5PPP/1Q1RRBK1 w - - 0 21
2qrrbk1/pp3pp1/5nnp/3pp3/3P4/1PP1N1Pb/PB1N1PBP/1QR1R1K1 w - - 1 21
q1rn2k1/1b2bppp/2p1pn2/8/1N1P4/4PNP1/3B1PBP/rR1Q2K1 w - - 0 21
r2q1rk1/p4p1p/1p2p1p1/3b2P1/2nP4/2P3Q1/P1B2PP1/R1B1R1K1 w - - 2 21
1r3rk1/1b1qbppp/4p3/1Rp1P3/3nB3/P1BP1N2/4QPPP/R5K1 w - - 1 21
2r3r1/3nkp2/bp1pp2p/p5p1/2Pp2PP/PP1BPP2/4K3/2R1B2R w - - 0 21
2bqnrk1/r2n3p/p2p4/2pPp1p1/2P1Pp2/PNN2P2/1R1QB1PP/5RK1 w - - 0 21
3rr1k1/1bp1qppp/1p1b1n2/pP1p4/3P4/PQN1PB2/1B3PPP/2RR2K1 w - - 0 21
1k1r3r/1b2bpp1/1q1p4/p2PpP1p/PpR1B3/1N3Q2/1PP3PP/2KR4 w - h6 0 21
r3rb2/1bqnnp1k/pp1pp1pP/8/P3PP1Q/1NN4R/1PPB2BP/5RK1 w - - 1 21
2r1r1k1/1p1bqpp1/p1n4p/3p3P/3N1PP1/P1P2B2/1P1Q1K2/6RR w - - 4 21
r4rk1/pp2q2p/1b4n1/4pb2/4Q3/1N3NP1/PPP3PP/2KRR3 w - - 2 21
r4rk1/ppqb1p1p/6p1/3P4/n1p1PP1P/2N5/PP1Q4/2K1RB1R w - - 1 21
r4rk1/ppq2p1p/2b3p1/4PR2/3p2Q1/2NP2P1/PP2P2P/R5K1 w - - 0 21
4r1k1/1p2ppbp/1n4p1/3p4/1B1P4/1P1bP1P1/5PBP/r1R1N1K1 w - - 0 21
1r3rk1/p4p1p/3npQp1/q1p5/2Pn1P2/2N3P1/P4PBP/1R3RK1 w - - 11 21
rr5k/4q1pp/3b1n2/Q3ppB1/1n1p4/P2N2P1/1P2PPNP/1R3RK1 w - - 1 21
r6r/1qpk2pp/p4n2/P7/8/2p1Q1P1/1PP2P1P/R4RK1 w - - 0 21
r3kb1r/1N1nn2p/1q2Q1p1/1p6/1p6/8/PP3PPP/R1B2RK1 w kq - 1 21
r4r1k/2p2ppp/p2b1q2/1p6/5P2/3QP3/PP1B2PP/R4R1K w - - 0 21
1r1qbr1k/6pp/2np4/p2Np1b1/R7/1PPB4/2N1QPPP/5RK1 w - - 4 21
r3r1k1/pp2qp1p/2pp2p1/4bP2/6P1/2P1BQ1P/PPP5/3R1RK1 w - - 3 21
r2q1rk1/p4bb1/1p1n1p1p/3p2p1/3P4/2NBB3/PPQ2PPP/2R1R1K1 w - - 0 21
r2qrnk1/1p3pp1/2n4p/p2pb3/8/PN3QBP/1P3PP1/RB2R1K1 w - - 0 21
r7/ppp1kppb/2p4p/4P3/5Pn1/2N3N1/PPPR1KPP/8 w - - 7 21
r1bb4/pp2k3/1np2p1p/2N1p1pn/1P2P3/PBN3B1/2P2PPP/2KR4 w - - 0 21
r3rbk1/1bqn1p1p/p2p2p1/1p6/3QP3/4B1NP/PPB1RPP1/R5K1 w - - 0 21
r3kn2/3qbpr1/n3p1p1/pp1pP1Np/2pP3P/2P2RQN/PP1B1PP1/R5K1 w q - 10 21
2rr1bk1/1pq2p1p/p1b1p1p1/n1P1P3/3B1P2/2PB2N1/PQ4PP/1R3RK1 w - - 2 21
3r2k1/pbpq1npp/1p6/n1P1p3/4P2P/2PBBN2/P1Q3P1/R5K1 w - - 3 21
r1b3k1/1pq1rpp1/2p2n1p/p5N1/P1BP4/4P3/1RQ2PPP/5RK1 w - - 0 21
1r1r2k1/p4pp1/n1bp1b1p/2p1p3/2P1P3/PqN3P1/1PN1QPBP/1R1R2K1 w - - 2 21
r3k3/1b1n2p1/p2ppnqr/1p6/3NP2Q/2N5/PPP3BP/4RRK1 w q - 3 21
1r2rbk1/pbn2pp1/1p1q3p/3pN3/3P1P2/PQN3P1/1P4BP/2R1R1K1 w - - 3 21
6k1/1bq2pPp/p5n1/1p2p3/2p1P1B1/2N1Q3/PPP3PP/3r2K1 w - - 0 21
2q1k2r/2r3pp/p1p1p3/1pb1P3/P1p1Q2P/2P3P1/5PB1/R4RK1 w k - 2 21
b1r1qrk1/p4ppp/1p1b4/3PN3/2p2B2/4Q3/PP3PPP/R2R2K1 w - - 2 21
r3k2r/1pqb2p1/p4p2/P2npP2/2pB2Bp/2P4P/2P1Q1P1/R4RK1 w kq - 0 21
2rr2k1/1b3ppp/p3p3/1p6/1P1B1Pnq/P2BP3/1Q4PP/3R1RK1 w - - 7 21
2rqk2r/1p1n1p2/p3p1p1/P2pP2p/1P1NbP2/2P1Q3/4B1PP/R2R2K1 w k - 1 21";

            let params = Parameters::default();
            let new_linesplit = positions.split("\n").collect::<Vec<&str>>();
            for line in new_linesplit {
                let position = GameState::from_fen(line);
                let evaluation = eval_game_state(&position);
                let trace_eval = evaluation.trace.collapse().evaluate(&params) as i16;
                //Rounding erros can make up for max 2 error (only 2 place where rounding can make a difference )
                if (evaluation.final_eval - trace_eval).abs() > 2 {
                    println!("{}", position.to_fen());
                    panic!(format!("{} != {}", evaluation.final_eval, trace_eval));
                }
            }
        }
    }
}
