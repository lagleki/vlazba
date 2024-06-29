use std::collections::HashMap;
use once_cell::sync::Lazy;
use serde_json;

type RafsiMap = HashMap<String, Vec<String>>;

static GISMU_RAFSI_LIST: Lazy<RafsiMap> = Lazy::new(|| {
    serde_json::from_str(include_str!("gismu_rafsi_list.json"))
        .expect("Failed to parse gismu_rafsi_list.json")
});

static GISMU_RAFSI_LIST_EXP: Lazy<RafsiMap> = Lazy::new(|| {
    serde_json::from_str(include_str!("gismu_rafsi_list_exp.json"))
        .expect("Failed to parse gismu_rafsi_list_exp.json")
});

static CMAVO_RAFSI_LIST: Lazy<RafsiMap> = Lazy::new(|| {
    serde_json::from_str(include_str!("cmavo_rafsi_list.json"))
        .expect("Failed to parse cmavo_rafsi_list.json")
});

static CMAVO_RAFSI_LIST_EXP: Lazy<RafsiMap> = Lazy::new(|| {
    serde_json::from_str(include_str!("cmavo_rafsi_list_exp.json"))
        .expect("Failed to parse cmavo_rafsi_list_exp.json")
});

pub fn get_gismu_rafsi_list() -> &'static RafsiMap {
    &GISMU_RAFSI_LIST
}

pub fn get_gismu_rafsi_list_exp() -> &'static RafsiMap {
    &GISMU_RAFSI_LIST_EXP
}

pub fn get_cmavo_rafsi_list() -> &'static RafsiMap {
    &CMAVO_RAFSI_LIST
}

pub fn get_cmavo_rafsi_list_exp() -> &'static RafsiMap {
    &CMAVO_RAFSI_LIST_EXP
}