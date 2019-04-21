use crate::board_representation::game_state::GameState;

pub struct UCIEngine<'a> {
    pub name: &'a str,
    pub author: &'a str,
    pub internal_state: GameState,
}

impl<'a> UCIEngine<'a> {
    pub fn standard() -> UCIEngine<'a> {
        UCIEngine {
            name: &"FabChess v1.0",
            author: &"Fabian von der Warth",
            internal_state: GameState::standard(),
        }
    }

    pub fn id_command(&self) {
        println!("id name {}", self.name);
        println!("id author {}", self.author);
    }
}
