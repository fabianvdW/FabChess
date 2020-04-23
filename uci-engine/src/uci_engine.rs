use core_sdk::board_representation::game_state::GameState;

pub struct UCIEngine<'a> {
    pub name: &'a str,
    pub author: &'a str,
    pub internal_state: GameState,
}

impl<'a> UCIEngine<'a> {
    pub fn standard() -> UCIEngine<'a> {
        UCIEngine {
            name: &"FabChessDev v1.14.1",
            author: &"Fabian von der Warth, Contributor: Erik Imgrund",
            internal_state: GameState::standard(),
        }
    }

    pub fn id_command(&self) {
        println!("id name {}", self.name);
        println!("id author {}", self.author);
    }
}
