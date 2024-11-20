use super::rng::Rng;


const SUPPORTED_CHARACTERS: usize = 32;
const NULL: usize = SUPPORTED_CHARACTERS - 3;
const START_OF: usize = SUPPORTED_CHARACTERS - 2;
const END_OF: usize = SUPPORTED_CHARACTERS - 1;

fn idx3(i: usize, j: usize, k: usize, l: usize) -> usize {
    return (i * SUPPORTED_CHARACTERS * SUPPORTED_CHARACTERS * SUPPORTED_CHARACTERS) + (j * SUPPORTED_CHARACTERS * SUPPORTED_CHARACTERS) + (k * SUPPORTED_CHARACTERS) + l
}

fn idx2(i: usize, j: usize, k: usize) -> usize {
    return (i * SUPPORTED_CHARACTERS * SUPPORTED_CHARACTERS) + (j * SUPPORTED_CHARACTERS) + k
}

fn idx1(i: usize, j: usize) -> usize {
    return (i * SUPPORTED_CHARACTERS) + j
}

pub struct MarkovChainSingleWordModel {
    model_o1: Vec<f32>,
    model_o2: Option<Vec<f32>>,
    model_o3: Option<Vec<f32>>
}

impl MarkovChainSingleWordModel {
    pub fn train(words: Vec<&str>, order: u8) -> MarkovChainSingleWordModel {
        assert!(order >= 1 && order <= 3, "Order must be between 1 and 3");
        let mut i_model: Vec<u32> = vec![0; SUPPORTED_CHARACTERS.pow(4)];
        for word in words {
            let mut li: [usize; 3] = [NULL, NULL, START_OF];
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
                    i_model[idx3(li[1], li[2], index, END_OF)] = i_model[idx3(li[1], li[2], index, END_OF)] + 1;
                }
                i_model[idx3(li[0], li[1], li[2], index)] = i_model[idx3(li[0], li[1], li[2], index)] + 1;
                li[0] = li[1];
                li[1] = li[2];
                li[2] = index;
            }
        }
        let mut model;
        match order {
            2 => model = MarkovChainSingleWordModel {
                model_o1: vec![0.0; SUPPORTED_CHARACTERS.pow(2)],
                model_o2: Some(vec![0.0; SUPPORTED_CHARACTERS.pow(3)]),
                model_o3: None
            },
            3 => model = MarkovChainSingleWordModel {
                model_o1: vec![0.0; SUPPORTED_CHARACTERS.pow(2)],
                model_o2: Some(vec![0.0; SUPPORTED_CHARACTERS.pow(3)]),
                model_o3: Some(vec![0.0; SUPPORTED_CHARACTERS.pow(4)])
            },
            _ => model = MarkovChainSingleWordModel {
                model_o1: vec![0.0; SUPPORTED_CHARACTERS.pow(2)],
                model_o2: None,
                model_o3: None
            }
        }
        // Normalize

        // i = last
        // j = second-to-last
        // k = third-to-last
        // l = current

        // TODO: How do I normalize the other ones???

        for i in 0..SUPPORTED_CHARACTERS {
            for j in 0..SUPPORTED_CHARACTERS {
                for k in 0..SUPPORTED_CHARACTERS {
                    let mut sum_o3 = 0;
                    for l in 0..SUPPORTED_CHARACTERS {
                        sum_o3 = sum_o3 + i_model[idx3(i, j, k, l)];
                    }
                    if let Some(model_o3) = &mut model.model_o3 {
                        for l in 0..SUPPORTED_CHARACTERS {
                            model_o3[idx3(i, j, k, l)] = i_model[idx3(i, j, k, l)] as f32 / sum_o3 as f32;
                        }
                    }
                    
                }
            }
        }
        return model
    }

    pub fn generate(&self, seed: u32, min_length: u8, max_length: u8) -> String {
        let mut rng = Rng::new(seed);
        let mut string = String::new();

        let mut history = [NULL, NULL, START_OF];

        for _ in 0..max_length {

            let mut char = NULL;

            let selected = rng.randf();
            let mut acc = 0.0;
            if let Some(model_o3) = &self.model_o3 {
                for i in 0..SUPPORTED_CHARACTERS {
                    acc = acc + model_o3[idx3(history[0], history[1], history[2], i)];
                    if acc > selected {
                        if i == END_OF && string.len() < min_length as usize {
                            continue
                        }
                        char = i;
                        break
                    }
                }
            }
            if char == NULL {
                if let Some(model_o2) = &self.model_o2 {
                    for i in 0..SUPPORTED_CHARACTERS {
                        acc = acc + model_o2[idx2(history[1], history[2], i)];
                        if acc > selected {
                            if i == END_OF && string.len() < min_length as usize {
                                continue
                            }
                            char = i;
                            break
                        }
                    }
                }
            }
            if char == NULL {
                for i in 0..SUPPORTED_CHARACTERS {
                    acc = acc + self.model_o1[idx1(history[2], i)];
                    if acc > selected {
                        if i == END_OF && string.len() < min_length as usize {
                            continue
                        }
                        char = i;
                        break
                    }
                }
            }

            match char {
                NULL => string.push('0'),
                START_OF => string.push('^'),
                END_OF => break,
                26 => string.push('ø'),
                27 => string.push('\''),
                28 => string.push('-'),
                _ => string.push(char::from_u32(char as u32 + 97).unwrap())
            }

            history[0] = history[1];
            history[1] = history[2];
            history[2] = char;

        }
        
        return string;
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {

        // let mo1 = MarkovChainSingleWordModel::train(vec!("john", "joe", "joseph", "jonny", "jon", "jonathan", "jasper"), 1);
        // assert_eq!(mo1.generate(0, 3, 10), "jasper");
        // assert_eq!(mo1.generate(10, 3, 10), "jon");

        // let mo2 = MarkovChainSingleWordModel::train(vec!("john", "joe", "joseph", "jonny", "jon", "jonathan", "jasper"), 2);
        // assert_eq!(mo2.generate(0, 3, 10), "jasper");
        // assert_eq!(mo2.generate(10, 3, 10), "jon");

        let mo3 = MarkovChainSingleWordModel::train(vec!("john", "joe", "joseph", "jonny", "jon", "jonathan", "jasper"), 3);
        assert_eq!(mo3.generate(0, 3, 10), "jasper");
        assert_eq!(mo3.generate(10, 3, 10), "jon");
    }
}