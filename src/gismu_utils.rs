use once_cell::sync::Lazy;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};

pub const C: &str = "bcdfgjklmnprstvxz";
pub const V: &str = "aeiou";

const VALID_CC_INITIALS: &[&str] = &[
    "bl", "br", "cf", "ck", "cl", "cm", "cn", "cp", "cr", "ct", "dj", "dr", "dz", "fl", "fr", "gl",
    "gr", "jb", "jd", "jg", "jm", "jv", "kl", "kr", "ml", "mr", "pl", "pr", "sf", "sk", "sl", "sm",
    "sn", "sp", "sr", "st", "tc", "tr", "ts", "vl", "vr", "xl", "xr", "zb", "zd", "zg", "zm", "zv",
];

const FORBIDDEN_CC: &[&str] = &["cx", "kx", "xc", "xk", "mz"];

const FORBIDDEN_CCC: &[&str] = &["ndj", "ndz", "ntc", "nts"];

const SIBILANT: &str = "cjsz";
const VOICED: &str = "bdgjvz";
const UNVOICED: &str = "cfkpstx";

pub static SIMILARITIES: Lazy<HashMap<char, &'static str>> = Lazy::new(|| {
    [
        ('b', "pv"),
        ('c', "js"),
        ('d', "t"),
        ('f', "pv"),
        ('g', "kx"),
        ('j', "cz"),
        ('k', "gx"),
        ('l', "r"),
        ('m', "n"),
        ('n', "m"),
        ('p', "bf"),
        ('r', "l"),
        ('s', "cz"),
        ('t', "d"),
        ('v', "bf"),
        ('x', "gk"),
        ('z', "js"),
    ]
    .iter()
    .cloned()
    .collect()
});

static VALID_CC_INITIALS_SET: Lazy<HashSet<&'static str>> =
    Lazy::new(|| VALID_CC_INITIALS.iter().cloned().collect());

static FORBIDDEN_CC_SET: Lazy<HashSet<&'static str>> =
    Lazy::new(|| FORBIDDEN_CC.iter().cloned().collect());

static FORBIDDEN_CCC_SET: Lazy<HashSet<&'static str>> =
    Lazy::new(|| FORBIDDEN_CCC.iter().cloned().collect());

static SIBILANT_SET: Lazy<HashSet<char>> = Lazy::new(|| SIBILANT.chars().collect());

static VOICED_SET: Lazy<HashSet<char>> = Lazy::new(|| VOICED.chars().collect());

static UNVOICED_SET: Lazy<HashSet<char>> = Lazy::new(|| UNVOICED.chars().collect());

pub fn get_string_value(key: char) -> &'static str {
    SIMILARITIES.get(&key).map_or(".", |&s| s)
}

pub fn language_weights() -> HashMap<&'static str, Vec<f32>> {
    [
        ("1985", vec![0.36, 0.16, 0.21, 0.11, 0.09, 0.07]),
        ("1987", vec![0.36, 0.156, 0.208, 0.116, 0.087, 0.073]),
        ("1994", vec![0.348, 0.194, 0.163, 0.123, 0.088, 0.084]),
        ("1995", vec![0.347, 0.196, 0.16, 0.123, 0.089, 0.085]),
        ("1999", vec![0.334, 0.195, 0.187, 0.116, 0.081, 0.088]),
    ]
    .iter()
    .cloned()
    .collect()
}

fn lcs_length(a: &str, b: &str) -> f32 {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let (m, n) = (a_chars.len(), b_chars.len());
    let mut dp = vec![0; n + 1];

    for i in 1..=m {
        let mut prev = 0;
        for j in 1..=n {
            let temp = dp[j];
            if a_chars[i - 1] == b_chars[j - 1] {
                dp[j] = prev + 1;
            } else {
                dp[j] = dp[j].max(dp[j - 1]);
            }
            prev = temp;
        }
    }

    dp[n] as f32
}

pub struct GismuGenerator {
    c: Vec<String>,
    v: Vec<String>,
    shape_strings: Vec<String>,
}

impl GismuGenerator {
    pub fn new(c: Vec<String>, v: Vec<String>, shape_strings: Vec<String>) -> Self {
        Self {
            c,
            v,
            shape_strings,
        }
    }

    pub fn iterator(&self) -> Vec<String> {
        self.shape_strings
            .par_iter()
            .flat_map(|shape_string| self.shape_iterator(shape_string))
            .collect()
    }

    fn shape_iterator(&self, shape_string: &str) -> Vec<String> {
        let shape = self.shape_for_string(shape_string);
        let validator = self.shape_validator(shape_string);

        shape
            .iter()
            .fold(vec![String::new()], |a, b| {
                a.into_iter()
                    .flat_map(|x| b.iter().map(move |y| x.clone() + y))
                    .collect()
            })
            .into_iter()
            .filter(|x| validator(x))
            .collect()
    }

    fn shape_for_string(&self, string: &str) -> Vec<&[String]> {
        string
            .chars()
            .map(|c| match c.to_ascii_lowercase() {
                'c' => &self.c[..],
                'v' => &self.v[..],
                _ => &[],
            })
            .collect()
    }
    
    fn shape_validator(&self, shape: &str) -> impl Fn(&str) -> bool {
        type Predicate = Box<dyn Fn(&str) -> bool + Send + Sync>;
    
        let predicates: Vec<Predicate> = shape
            .chars()
            .zip(shape.chars().skip(1))
            .enumerate()
            .filter_map(|(i, (c1, c2))| {
                if c1.to_ascii_lowercase() == 'c' && c2.to_ascii_lowercase() == 'c' {
                    let mut p: Vec<Predicate> = vec![Box::new(self.validator_for_cc(i))];
                    if shape.chars().nth(i + 2) == Some('c') {
                        p.push(Box::new(self.validator_for_ccc(i)));
                    }
                    if i > 0 && shape[i..].starts_with("ccvcv") {
                        p.push(Box::new(self.invalidator_for_initial_cc(i)));
                    }
                    Some(p)
                } else {
                    None
                }
            })
            .flatten()
            .collect();
    
        move |x: &str| predicates.iter().all(|p| p(x))
    }

    fn validator_for_cc(&self, i: usize) -> impl Fn(&str) -> bool {
        move |x: &str| {
            if i == 0 {
                VALID_CC_INITIALS_SET.contains(&x[..2])
            } else {
                let j = i + 1;
                let c1 = x.chars().nth(i).unwrap();
                let c2 = x.chars().nth(j).unwrap();

                !(c1 == c2
                    || (VOICED_SET.contains(&c1) && UNVOICED_SET.contains(&c2))
                    || (UNVOICED_SET.contains(&c1) && VOICED_SET.contains(&c2))
                    || (SIBILANT_SET.contains(&c1) && SIBILANT_SET.contains(&c2))
                    || FORBIDDEN_CC_SET.contains(&x[i..=j]))
            }
        }
    }

    fn validator_for_ccc(&self, i: usize) -> impl Fn(&str) -> bool {
        move |x| !FORBIDDEN_CCC_SET.contains(&x[i..=i + 2])
    }

    fn invalidator_for_initial_cc(&self, i: usize) -> impl Fn(&str) -> bool {
        move |x| !VALID_CC_INITIALS_SET.contains(&x[i..=i + 1])
    }
}

pub struct GismuScorer<'a> {
    input_words: &'a [String],
    weights: &'a [f32],
}

impl<'a> GismuScorer<'a> {
    pub fn new(input_words: &'a [String], weights: &'a [f32]) -> Self {
        Self {
            input_words,
            weights,
        }
    }

    fn compute_score(&self, candidate: &str) -> (f32, Vec<f32>) {
        let similarity_scores: Vec<f32> = self
            .input_words
            .iter()
            .map(|word| {
                let lcs_len = lcs_length(candidate, word);
                let score = match lcs_len {
                    0.0 | 1.0 => 0.0,
                    2.0 => self.score_dyad_by_pattern(candidate, word),
                    _ => lcs_len,
                };
                score / word.len() as f32
            })
            .collect();

        let weighted_sum = self.calculate_weighted_sum(&similarity_scores);
        (weighted_sum, similarity_scores)
    }

    pub fn compute_score_with_name<'b>(
        &self,
        candidate: &'b String,
    ) -> (f32, &'b String, Vec<f32>) {
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

    fn calculate_weighted_sum(&self, scores: &[f32]) -> f32 {
        scores
            .iter()
            .zip(self.weights)
            .map(|(&score, &weight)| score * weight)
            .sum()
    }
}

pub struct GismuMatcher<'a> {
    gismus: &'a [String],
    stem_length: usize,
}

impl<'a> GismuMatcher<'a> {
    pub fn new(gismus: &'a [String], stem_length: Option<usize>) -> Self {
        Self {
            gismus,
            stem_length: stem_length.unwrap_or(4),
        }
    }

    pub fn find_similar_gismu(&self, candidate: &str) -> Option<String> {
        let candidate = candidate.trim_end();
        let patterns: Vec<String> = candidate
            .chars()
            .map(|c| get_string_value(c).to_string())
            .collect();

        self.gismus
            .iter()
            .find(|word| self.match_gismu(word, candidate, &patterns))
            .cloned()
    }

    fn match_gismu(&self, gismu: &str, candidate: &str, structural_patterns: &[String]) -> bool {
        self.match_stem(gismu, candidate)
            || self.match_structure(gismu, candidate, structural_patterns)
    }

    fn match_stem(&self, gismu: &str, candidate: &str) -> bool {
        candidate.len() >= self.stem_length && gismu.starts_with(&candidate[..self.stem_length])
    }

    fn match_structure(
        &self,
        gismu: &str,
        candidate: &str,
        structural_patterns: &[String],
    ) -> bool {
        let common_len = candidate.len().min(gismu.len());
        (0..common_len).any(|i| {
            self.strings_match_except(gismu, candidate, i, common_len)
                && self.match_structural_pattern(&gismu[i..i + 1], &structural_patterns[i])
        })
    }

    fn strings_match_except(&self, x: &str, y: &str, i: usize, j: usize) -> bool {
        x[..i] == y[..i] && x[(i + 1)..j] == y[(i + 1)..j]
    }

    fn match_structural_pattern(&self, letter: &str, pattern: &str) -> bool {
        pattern == "." || pattern.contains(letter)
    }
}
