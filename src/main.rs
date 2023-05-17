use clap::__derive_refs::once_cell;
use clap::{Arg, Command};
use std::collections::HashSet;
use std::fs::File;
use std::io::{stderr, BufRead, BufReader, Write};
use std::{println, process};
use regex::Regex;

use rayon::prelude::*;

mod gismu_utils;
use gismu_utils::{language_weights, GismuGenerator, GismuMatcher, GismuScorer, C, V};

const VERSION: &str = "v0.5";

use once_cell::sync::Lazy;

static DEFAULT_WEIGHTS_STR: Lazy<String> = Lazy::new(|| {
    language_weights()
        .get("1985")
        .unwrap()
        .to_vec()
        .iter()
        .map(|&weight| weight.to_string())
        .collect::<Vec<String>>()
        .join(",")
});

fn log(msg: &str) {
    writeln!(&mut stderr(), "{}", msg).unwrap();
}

fn split_string_to_letters(s: &str) -> Vec<String> {
    s.chars().map(|c| c.to_string()).collect()
}

fn generate_weights(weights_str: String) -> Vec<f32> {
    let opt_str = "weights";
    let re = Regex::new(r"(\d{4}|finprims)$").unwrap();
    let weights: Vec<f32>;
    if re.is_match(&weights_str) {
        if let Some(found_weights) = language_weights().get(weights_str.as_str()) {
            log(&format!("Using language weights from {}...", weights_str));
            weights = found_weights.to_vec();
        } else {
            panic!("No weights registered for {}", weights_str);
        }
    } else {
        weights = weights_str
        .split(',')
        .map(|x| {
            let weight = x.trim().parse::<f32>();
            if weight.is_err() || weight.as_ref().unwrap() <= &0.0 {
                panic!("Values for {} must be numbers greater than zero", opt_str);
            }
            weight.unwrap()
        })
        .collect::<Vec<f32>>();
        if weights.len() < 2 {
            panic!("{} must include at least 2 values", opt_str);
        }
    }
    weights
}


fn main() {
    let matches = Command::new("Optimized Gismu Generator")
        .version(VERSION)
        .arg(Arg::new("words").help("Input words"))
        .arg(
            Arg::new("all-letters")
                .short('a')
                .long("all-letters")
                .required(false)
                .num_args(0)
                .help("Use all letters"),
        )
        .arg(
            Arg::new("shapes")
                .short('s')
                .long("shapes")
                .default_value("ccvcv,cvccv")
                .help("Shapes for gismu candidates"),
        )
        .arg(
            Arg::new("weights")
                .short('w')
                .long("weights")
                .default_value(DEFAULT_WEIGHTS_STR.as_str())
                .help("Weights for input words"),
        )
        .arg(
            Arg::new("deduplicate")
                .short('d')
                .long("deduplicate")
                .help("Path to gismu list for deduplication"),
        )
        .get_matches();

    let binding = "".to_string();
    let words: Vec<String> = matches
        .get_one::<String>("words")
        .unwrap_or(&binding)
        .split(' ')
        .collect::<Vec<&str>>()
        .iter()
        .map(|s| s.to_string())
        .collect();
    let all_letters = *matches.get_one::<bool>("all-letters").unwrap_or(&false);
    let binding = "".to_string();
    let shapes: Vec<String> = matches
        .get_one::<String>("shapes")
        .unwrap_or(&binding)
        .split(',')
        .map(|shape| shape.trim().to_string())
        .collect();
    let weights = generate_weights(matches
        .get_one::<String>("weights")
        .unwrap_or(&"".to_string()).clone());

    let gismu_list_path = matches.get_one::<String>("deduplicate");

    if let Err(e) = validate_words(&words, &weights) {
        log(&e);
        process::exit(1);
    }

    let (c, v) = if all_letters {
        (split_string_to_letters(C), split_string_to_letters(V))
    } else {
        letters_for_words(&words)
    };
    log(&format!(
        "Using letters {} and {}.",
        c.iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(","),
        v.iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(",")
    ));
    let shapes: Vec<String> = shapes.clone();
    let candidate_iterator = GismuGenerator::new(c, v, shapes);
    let candidates: Vec<String> = candidate_iterator.iterator();
    log(&format!("{} candidates generated.", candidates.len()));

    let scorer = GismuScorer::new(&words, &weights);

    let mut scores: Vec<(f32, &String, Vec<f32>)> = candidates
        .par_iter()
        .map(|candidate| scorer.compute_score_with_name(candidate))
        .collect();

    scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    log("\n10 first gismu candidates are:\n");
    for record in &scores[..10] {
        log(&format!("{:?}", record));
    }

    if let Some(gismu_list_path) = gismu_list_path {
        log("Reading list of gismu... ");
        let gismus = read_gismu_list(gismu_list_path).unwrap();
        let matcher = GismuMatcher::new(&gismus, None);
        log("Excluding candidates similar to existing gismu...");
        let candidate = deduplicate_candidates(&matcher, &scores);
        if let Some(candidate) = candidate {
            log("The winner is....");
            println!("{}", candidate.to_uppercase());
        } else {
            log("No suitable candidates found.");
        }
    }
}

fn letters_for_words(words: &[String]) -> (Vec<String>, Vec<String>) {
    let word_set: HashSet<char> = words.iter().flat_map(|word| word.chars()).collect();

    (
        C.chars()
            .filter(|&c| word_set.contains(&c))
            .map(|c| c.to_string())
            .collect(),
        V.chars()
            .filter(|&c| word_set.contains(&c))
            .map(|c| c.to_string())
            .collect(),
    )
}

fn deduplicate_candidates(
    matcher: &GismuMatcher,
    scores: &Vec<(f32, &String, Vec<f32>)>,
) -> Option<String> {
    scores
        .iter()
        .find_map(|(_, candidate, _)| {
            matcher
                .find_similar_gismu(candidate)
                .map(|gismu| {
                    log(&format!(
                        "Candidate '{}' too much like gismu '{}'.",
                        candidate, gismu
                    ));
                })
                .map(|_| (*candidate.clone()).to_string())
        })
}

fn validate_words(words: &Vec<String>, weights: &Vec<f32>) -> Result<(), String> {
    let weight_count = weights.len();
    if words.len() != weight_count {
        return Err(format!("Expected {} words as input", weight_count));
    }
    for word in words {
        if word.len() < 2 {
            return Err("Input words must be at least two letters long".to_string());
        }
    }
    Ok(())
}

fn read_gismu_list(path: &str) -> Result<Vec<String>, std::io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let gismus: Result<Vec<String>, _> = reader.lines().collect();
    gismus
}
