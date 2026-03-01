#![allow(dead_code)]

use std::fs;
use serde::Deserialize;
use crate::core::suffix_fsm::*;
// Kullanılmayan `crate::core::root_trie::*` içe aktarımı silindi. (Uyarıyı çözer)
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
struct RawRoot { 
    word: String, 
    flags: Vec<String>,
    #[serde(default = "default_domain")] // YENİ: Domain yazmıyorsa GENERAL olsun
    domain: String, 
}

fn default_domain() -> String {
    "GENERAL".to_string()
}

#[derive(Deserialize)]
struct AbbrDatabase { abbreviations: std::collections::HashMap<String, String> }

pub fn load_abbreviations_from_json(file_path: &str) -> std::collections::HashMap<String, String> {
    if let Ok(data) = std::fs::read_to_string(file_path) {
        if let Ok(db) = serde_json::from_str::<AbbrDatabase>(&data) {
            println!("[SİSTEM] {} adet Kısaltma Sözlüğü yüklendi.", db.abbreviations.len());
            return db.abbreviations;
        }
    }
    std::collections::HashMap::new()
}

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
            "IS_TRANSITIVE" => bitmask |= crate::core::root_trie::IS_TRANSITIVE,     // YENİ EKLENDİ
            "IS_INTRANSITIVE" => bitmask |= crate::core::root_trie::IS_INTRANSITIVE, // YENİ EKLENDİ
            "IS_NOUN" => {}, // MÜHÜR: İsim bayrağı uyarı vermesin!
            "IS_ADJECTIVE" => {}, // YENİ EKLENDİ: Uyarı vermesin!
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
        // HATA ÇÖZÜLDÜ: get_domain_fast'te tanımlı olan &raw.domain yerine `insert` için doğru imzayı kullanıyoruz
        // (Çünkü root_trie'de `insert` metodu artık 3. parametre olarak `&str` domain alıyor)
        engine.root_dict.insert(&raw.word, dna, &raw.domain); 
    }
    println!("[SİSTEM] Kökler Trie ağına yüklendi.");
}

// ==========================================
// YENİ EKLENDİ: DOMAIN MATRIX YÜKLEYİCİ
// ==========================================
pub fn load_domain_matrix_from_json(engine: &mut MorphEngine, file_path: &str) {
    let data = fs::read_to_string(file_path).expect("[KRİTİK HATA] domain_matrix.json okunamadı!");
    
    // JSON yapısı doğrudan bir Map ("ANA_DOMAIN" -> ["KARDEŞ1", "KARDEŞ2"]) olduğu için 
    // ekstra bir ara struct'a ihtiyaç duymadan doğrudan Deserialize ediyoruz.
    let matrix: std::collections::HashMap<String, Vec<String>> = serde_json::from_str(&data)
        .expect("[KRİTİK HATA] Domain Matrix JSON formatı bozuk!");
    
    engine.domain_matrix = matrix;
    
    println!("[SİSTEM] Semantik Domain Matrisi (Kardeş Alanlar) Engine'e yüklendi.");
}