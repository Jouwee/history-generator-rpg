use super::rng::Rng;


const SUPPORTED_CHARACTERS: usize = 31;
const START_OF: usize = SUPPORTED_CHARACTERS - 2;
const END_OF: usize = SUPPORTED_CHARACTERS - 1;
pub struct MarkovChainSingleWordModel {
    model: [[f32; SUPPORTED_CHARACTERS]; SUPPORTED_CHARACTERS]
}

impl MarkovChainSingleWordModel {
    pub fn train(words: Vec<&str>, order: u8) -> MarkovChainSingleWordModel {
        let mut i_model = [[0; SUPPORTED_CHARACTERS]; SUPPORTED_CHARACTERS];
        for word in words {
            let mut last_index: usize = 0;
            for (i, character) in word.chars().enumerate() {
                let character = character.to_ascii_lowercase();
                let index: usize;
                match character {
                    'ø' => index = 26,
                    '\'' => index = 27,
                    '-' => index = 28,
                    _ => {
                        assert!(character as usize >= 97, "fail, {character} {}", character as usize);
                        index = character as usize - 97;
                    }
                }
                if i == word.len() - 1 {
                    i_model[index][END_OF] = i_model[index][END_OF] + 1;
                }
                if i == 1 {
                    i_model[START_OF][index] = i_model[START_OF][index] + 1;
                }
                if i >= 1 {
                    i_model[last_index][index] = i_model[last_index][index] + 1;
                }
                last_index = index;
            }
        }
        let mut model = MarkovChainSingleWordModel {
            model: [[0.0; SUPPORTED_CHARACTERS]; SUPPORTED_CHARACTERS]
        };
        // Normalize
        for i in 0..SUPPORTED_CHARACTERS {
            let mut sum = 0;
            for count in i_model[i] {
                sum = sum + count;
            }
            for j in 0..SUPPORTED_CHARACTERS {
                model.model[i][j] = i_model[i][j] as f32 / sum as f32;
            }
        }
        return model
    }

    pub fn generate(&self, seed: u32, min_length: u8, max_length: u8) -> String {
        let mut rng = Rng::new(seed);
        let mut string = String::new();

        let mut char = START_OF;

        for _ in 0..max_length {
            let selected = rng.randf();
            let mut acc = 0.0;
            for i in 0..SUPPORTED_CHARACTERS {
                acc = acc + self.model[char][i];
                if acc > selected {
                    if i == END_OF && string.len() < min_length as usize {
                        continue
                    }
                    char = i;
                    break
                }
            }

            match char {
                END_OF => break,
                26 => string.push('ø'),
                27 => string.push('\''),
                28 => string.push('-'),
                _ => string.push(char::from_u32(char as u32 + 97).unwrap())
            }
        }
        
        return string;
    }

}