use serde_json::Value;
use std::collections::HashMap;

pub const C: &str = "bcdfgjklmnprstvxz";
pub const V: &str = "aeiou";

const VALID_CC_INITIALS: [&str; 48] = [
    "bl", "br", "cf", "ck", "cl", "cm", "cn", "cp", "cr", "ct", "dj", "dr", "dz", "fl", "fr", "gl",
    "gr", "jb", "jd", "jg", "jm", "jv", "kl", "kr", "ml", "mr", "pl", "pr", "sf", "sk", "sl", "sm",
    "sn", "sp", "sr", "st", "tc", "tr", "ts", "vl", "vr", "xl", "xr", "zb", "zd", "zg", "zm", "zv",
];

const FORBIDDEN_CC: [&str; 5] = ["cx", "kx", "xc", "xk", "mz"];

const FORBIDDEN_CCC: [&str; 4] = ["ndj", "ndz", "ntc", "nts"];

const SIBILANT: &str = "cjsz";
const VOICED: &str = "bdgjvz";
const UNVOICED: &str = "cfkpstx";

pub fn similarities() -> Value {
    serde_json::json!({
        "b":"pv",
        "c":"js",
        "d":"t",
        "f":"pv",
        "g":"kx",
        "j":"cz",
        "k":"gx",
        "l":"r",
        "m":"n",
        "n":"m",
        "p":"bf",
        "r":"l",
        "s":"cz",
        "t":"d",
        "v":"bf",
        "x":"gk",
        "z":"js"
    })
}

pub fn get_string_value<'a>(data: &'a Value, key: &'a str) -> String {
    serde_json::to_string(&data.get(key)).unwrap_or(".".to_string())
}

pub fn language_weights() -> HashMap<String, Vec<f32>> {
    let mut weights = HashMap::new();
    weights.insert("1985".to_string(), vec![0.36, 0.16, 0.21, 0.11, 0.09, 0.07]);
    weights.insert("1987".to_string(), vec![0.36, 0.156, 0.208, 0.116, 0.087, 0.073]);
    weights.insert("1994".to_string(), vec![0.348, 0.194, 0.163, 0.123, 0.088, 0.084]);
    weights.insert("1995".to_string(), vec![0.347, 0.196, 0.16, 0.123, 0.089, 0.085]);
    weights.insert("1999".to_string(), vec![0.334, 0.195, 0.187, 0.116, 0.081, 0.088]);
    weights
}

fn lcs_length(a: &str, b: &str) -> f32 {
    let w = b.len() + 1;
    let h = a.len() + 1;
    let mut matrix = vec![0; w * h];

    for (ix, x) in a.chars().enumerate() {
        for (iy, y) in b.chars().enumerate() {
            let i = ix * w + iy;
            matrix[i + w + 1] = if x == y {
                matrix[i] + 1
            } else {
                matrix[i + 1].max(matrix[i + w])
            };
        }
    }

    matrix.last().cloned().unwrap() as f32
}

fn xadd(a: &Vec<String>, b: &Vec<String>) -> Vec<String> {
    let mut result = Vec::new();
    for x in a {
        for y in b {
            result.push(x.to_owned() + y.to_owned().as_str());
        }
    }
    result.to_owned()
}

pub struct GismuGenerator {
    c: Vec<String>,
    v: Vec<String>,
    shape_strings: Vec<String>,
}

impl GismuGenerator {
    pub fn new(c: Vec<String>, v: Vec<String>, shape_strings: Vec<String>) -> GismuGenerator {
        GismuGenerator {
            c,
            v,
            shape_strings,
        }
    }

    pub fn iterator(&self) -> Vec<String> {
        let iterators: Vec<Vec<String>> = self
            .shape_strings
            .clone()
            .iter()
            .map(|shape_string| self.shape_iterator(shape_string.clone()))
            .collect();
        iterators.into_iter().flatten().collect::<Vec<String>>()
    }

    fn shape_iterator(&self, shape_string: String) -> Vec<String> {
        let shape = self.shape_for_string(shape_string.clone());
        let validator = self.shape_validator(shape_string.clone());
        shape
            .iter()
            .fold(vec![String::new()], |a, b| xadd(&a, &b))
            .into_iter()
            .filter(|x| validator(x.clone().as_str()))
            .collect()
    }

    fn shape_for_string(&self, string: String) -> Vec<Vec<String>> {
        let mut shape = Vec::new();
        for letter in string.chars().map(|c| c.to_ascii_lowercase()) {
            if letter == 'c' {
                shape.push(self.c.clone());
            } else if letter == 'v' {
                shape.push(self.v.clone());
            }
        }
        shape
    }

    fn shape_validator(&self, shape: String) -> impl Fn(&str) -> bool {
        let mut predicates: Vec<Box<dyn Fn(&str) -> bool>> = Vec::new();
        let shape_chars = shape.chars().collect::<Vec<char>>();
        let slen = shape_chars.len();
        for i in 0..slen - 1 {
            let c = shape_chars[i].to_ascii_lowercase();
            if c == shape_chars[i + 1] && c == 'c' {
                predicates.push(Box::new(self.validator_for_cc(i)));
                if i < slen - 2 && shape_chars[i + 2] == 'c' {
                    predicates.push(Box::new(self.validator_for_ccc(i)));
                }
                if 0 < i && i < slen - 4 && shape_chars[i..i + 5] == ['c', 'c', 'v', 'c', 'v'] {
                    predicates.push(Box::new(self.invalidator_for_initial_cc(i)));
                }
            }
        }
        self.validator_for_predicates(predicates)
    }

    fn validator_for_cc(&self, i: usize) -> impl Fn(&str) -> bool {
        move |x: &str| {
            if i == 0 {
                VALID_CC_INITIALS.contains(&&x[..2])
            } else {
                let j = i + 1;
                let forbidden_cc = FORBIDDEN_CC.to_vec();
                let c1 = x.chars().nth(i).unwrap();
                let c2 = x.chars().nth(j).unwrap();
    
                !(c1 == c2
                    || (VOICED.contains(c1) && UNVOICED.contains(c2))
                    || (UNVOICED.contains(c1) && VOICED.contains(c2))
                    || (SIBILANT.contains(c1) && SIBILANT.contains(c2))
                    || forbidden_cc.contains(&&x[i..=j]))
            }
        }
    }
    
    fn validator_for_ccc(&self, i: usize) -> impl Fn(&str) -> bool {
        let forbidden_ccc: Vec<&str> = FORBIDDEN_CCC.iter().copied().collect();
        move |x| !forbidden_ccc.contains(&&x[i..=i + 2])
    }
    
    fn invalidator_for_initial_cc(&self, i: usize) -> impl Fn(&str) -> bool {
        let valid_cc_initials: Vec<&str> = VALID_CC_INITIALS.iter().copied().collect();
        move |x| !valid_cc_initials.contains(&&x[i..=i + 2])
    }
    
    fn validator_for_predicates(
        &self,
        predicates: Vec<impl Fn(&str) -> bool>,
    ) -> impl Fn(&str) -> bool {
        move |x: &str| predicates.iter().all(|p| p(x))
    }
    
}

pub struct GismuScorer<'a> {
    input_words: &'a Vec<String>,
    weights: &'a Vec<f32>,
}

impl GismuScorer<'_> {
    pub fn new<'a>(input_words: &'a Vec<String>, weights: &'a Vec<f32>) -> GismuScorer<'a> {
        GismuScorer {
            input_words,
            weights,
        }
    }

    fn compute_score(&self, candidate: &String) -> (f32, Vec<f32>) {
        let similarity_scores = self.input_words.iter().map(|word| {
            let lcs_len = lcs_length(candidate, word);
            let score = if lcs_len < 2.0 {
                0.0
            } else if lcs_len == 2.0 {
                self.score_dyad_by_pattern(candidate, word)
            } else {
                lcs_len
            };
            score / word.len() as f32
        }).collect();
    
        let weighted_sum = self.calculate_weighted_sum(&similarity_scores, self.weights);
        (weighted_sum, similarity_scores)
    }
    
    pub fn compute_score_with_name<'a>(&'a self, candidate: &'a String) -> (f32, &'a String, Vec<f32>) {
        let (weighted_sum, similarity_scores) = self.compute_score(candidate);
        (weighted_sum, candidate, similarity_scores)
    }
    
    fn score_dyad_by_pattern(&self, candidate: &str, input_word: &str) -> f32 {
        let l = candidate.len();
        let iw02: String = input_word.chars().step_by(2).collect();
        let iw12: String = input_word.chars().skip(1).step_by(2).collect();
    
        let mut score = 0.0;
    
        for i in 0..(l - 2) {
            let dyad = &candidate[i..(i + 2)];
            if iw02.contains(dyad) || iw12.contains(dyad) {
                score = 2.0;
                break;
            }
        }
    
        if score == 0.0 {
            for i in 0..(l - 1) {
                let dyad = &candidate[i..(i + 2)];
                if input_word.contains(dyad) {
                    score = 2.0;
                    break;
                }
            }
        }
    
        score
    }

    fn calculate_weighted_sum(&self, scores: &Vec<f32>, weights: &Vec<f32>) -> f32 {
        scores
            .iter()
            .zip(weights.iter())
            .map(|(&score, &weight)| score * weight)
            .sum()
    }
}

pub struct GismuMatcher<'a> {
    gismus: &'a Vec<String>,
    stem_length: usize,
}

impl GismuMatcher<'_> {
    pub fn new<'a>(gismus: &'a Vec<String>, stem_length: Option<usize>) -> GismuMatcher<'a> {
        GismuMatcher {
            gismus,
            stem_length: stem_length.unwrap_or(4),
        }
    }
    
    pub fn find_similar_gismu(&self, candidate: &str) -> Option<String> {
        let candidate = candidate.trim_end();
        let patterns_: Vec<String> = candidate
            .chars()
            .map(|c| get_string_value(&similarities(), c.to_string().as_str()))
            .collect();

        let patterns: Vec<&str> = patterns_.iter().map(|s| s.as_str()).collect();

        let mut gismu: Option<String> = None;

        for word in self.gismus.clone() {
            let found_match = self.match_gismu(&word, candidate, &patterns);
            if found_match {
                gismu = Some(word);
                break;
            }
        }

        gismu
    }

    fn match_gismu(&self, gismu: &str, candidate: &str, structural_patterns: &Vec<&str>) -> bool {
        self.match_stem(gismu, candidate)
            || self.match_structure(gismu, candidate, structural_patterns)
    }

    fn match_stem(&self, gismu: &str, candidate: &str) -> bool {
        candidate.len() >= self.stem_length
            && &candidate[..self.stem_length] == &gismu[..self.stem_length]
    }

    fn match_structure(
        &self,
        gismu: &str,
        candidate: &str,
        structural_patterns: &Vec<&str>,
    ) -> bool {
        let common_len = candidate.len().min(gismu.len());
        (0..common_len).any(|i| {
            self.strings_match_except(gismu, candidate, i, common_len)
                && self.match_structural_pattern(
                    gismu.chars().nth(i).unwrap().to_string().as_str(),
                    structural_patterns[i],
                )
        })
    }

    fn strings_match_except(&self, x: &str, y: &str, i: usize, j: usize) -> bool {
        x[..i] == y[..i] && x[(i + 1)..j] == y[(i + 1)..j]
    }

    fn match_structural_pattern(&self, letter: &str, pattern: &str) -> bool {
        if pattern == "." {
            return true;
        }
        pattern.contains(letter)
    }
}
