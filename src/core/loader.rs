#![allow(dead_code)]

use std::fs;
use serde::Deserialize;
use crate::core::suffix_fsm::*;
use crate::core::root_trie::*;
use crate::core::morph_engine::MorphEngine;

// 1. Bitmask Sabitlerine BUFFER_I'yı ekle (Eğer yoksa dosyanın başına ekle)
pub const BUFFER_I: u16 = 1 << 6; // Örnek olarak uygun bir bit seç

#[derive(Deserialize)]
struct SuffixDatabase { suffixes: Vec<RawSuffix> }

#[derive(Deserialize)]
struct RawSuffix {
    id: String, 
    base_form: String, 
    canonical_form: Option<String>, // YENİ: Makro Eklerin resmi genişlemesi!
    flags: Vec<String>,
    input_states: Vec<MorphState>, // YENİ: Bu ek hangi duraklardan tren alabilir?
    output_state: MorphState,
}

#[derive(Deserialize)]
struct RootDatabase { roots: Vec<RawRoot> }

#[derive(Deserialize)]
struct RawRoot { word: String, flags: Vec<String> }

pub fn parse_flags(flags: &[String]) -> u16 {
    let mut bitmask = 0;
    for f in flags {
        match f.as_str() {
            // Suffix Flags
            "HARMONY_FOUR_WAY" => bitmask |= crate::core::suffix_fsm::HARMONY_FOUR_WAY,
            "HARMONY_TWO_WAY" => bitmask |= crate::core::suffix_fsm::HARMONY_TWO_WAY,
            "BUFFER_S" => bitmask |= crate::core::suffix_fsm::BUFFER_S,
            "BUFFER_N" => bitmask |= crate::core::suffix_fsm::BUFFER_N,
            "BUFFER_Y" => bitmask |= crate::core::suffix_fsm::BUFFER_Y,
            "BUFFER_I" => bitmask |= BUFFER_I, // YENİ EKLENDİ!
            "MUT_D_T" => bitmask |= crate::core::suffix_fsm::MUT_D_T,
            "MUT_C_C" => bitmask |= crate::core::suffix_fsm::MUT_C_C,
            "MUT_G_K" => bitmask |= crate::core::suffix_fsm::MUT_G_K,
            "CAUSES_SOFTENING" => bitmask |= crate::core::suffix_fsm::CAUSES_SOFTENING,
            "ADDS_PRON_N" => bitmask |= crate::core::suffix_fsm::ADDS_PRON_N,
            "DROPS_INITIAL_I" => bitmask |= crate::core::suffix_fsm::DROPS_INITIAL_I,
            
            // Root Flags
            "DROPS_VOWEL" => bitmask |= crate::core::root_trie::DROPS_VOWEL,
            "TAKES_THIN_SUFFIX" => bitmask |= crate::core::root_trie::TAKES_THIN_SUFFIX,
            "IS_PROPER_NOUN" => bitmask |= crate::core::root_trie::IS_PROPER_NOUN,
            "IS_VERB" => bitmask |= crate::core::root_trie::IS_VERB,
            "RESISTS_SOFTENING" => bitmask |= crate::core::root_trie::RESISTS_SOFTENING,
            "IS_NOUN" => {}, // MÜHÜR: İsim bayrağı uyarı vermesin!
            _ => println!("[UYARI] Bilinmeyen bayrak: {}", f),
        }
    }
    bitmask
}

pub fn load_fsm_from_json(engine: &mut MorphEngine, file_path: &str) {
    let data = fs::read_to_string(file_path).expect("[KRİTİK HATA] suffixes.json okunamadı!");
    let db: SuffixDatabase = serde_json::from_str(&data).expect("[KRİTİK HATA] Suffix JSON bozuk!");
    
    let mut loaded_count = 0;
    for raw in db.suffixes {
        let dna = parse_flags(&raw.flags);
        // allowed_next argümanını kaldırdık, JSON mimarisi artık çok daha akıllı.
        let node = SuffixNode {
            id: raw.id.clone(),
            base_form: raw.base_form.clone(),
            canonical_form: raw.canonical_form.clone(), // YENİ EKLENDİ
            flags: dna,
            output_state: raw.output_state.clone(),
            allowed_next: vec![],
        };
        
        // EFSANEVİ DÖNGÜ: İf-Else yok. JSON'daki giriş duraklarına doğrudan ray döşer.
        for input_state in raw.input_states {
            engine.register_suffix_route(input_state, node.clone());
        }
        loaded_count += 1;
    }
    println!("[SİSTEM] {} adet ek FSM ağına dinamik olarak yüklendi.", loaded_count);
}

pub fn load_roots_from_json(engine: &mut MorphEngine, file_path: &str) {
    let data = fs::read_to_string(file_path).expect("[KRİTİK HATA] roots.json okunamadı!");
    let db: RootDatabase = serde_json::from_str(&data).expect("[KRİTİK HATA] Root JSON bozuk!");
    for raw in db.roots {
        let dna = parse_flags(&raw.flags);
        engine.root_dict.insert(&raw.word, dna);
    }
    println!("[SİSTEM] Kökler Trie ağına yüklendi.");
}