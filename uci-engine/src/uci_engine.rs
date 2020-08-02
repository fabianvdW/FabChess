use core_sdk::board_representation::game_state::GameState;

pub struct UCIEngine<'a> {
    pub name: &'a str,
    pub author: &'a str,
    pub contributors: &'a [&'a str],
    pub internal_state: GameState,
}

impl<'a> UCIEngine<'a> {
    pub fn standard() -> UCIEngine<'a> {
        UCIEngine {
            name: {
                #[cfg(all(target_arch = "x86_64", target_feature = "bmi2"))]
                {
                    &"FabChess v1.15 BMI2"
                }
                #[cfg(not(all(target_arch = "x86_64", target_feature = "bmi2")))]
                {
                    &"FabChess v1.15"
                }
            },
            author: &"Fabian von der Warth",
            contributors: &["Erik Imgrund", "Marcin Mielniczuk"],
            internal_state: GameState::standard(),
        }
    }

    pub fn id_command(&self) {
        println!("id name {}", self.name);
        println!("id author {}", self.author);
        println!("id contributors {}", self.contributors.join(", "))
    }
}
