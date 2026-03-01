#![allow(dead_code)]

use std::collections::BTreeMap;
use crate::core::distance::char_substitution_cost; // Ceza matrisini içeri alıyoruz

// 16-Bit Kök Bayrakları (Aynen kalıyor)
pub const DROPS_VOWEL: u16            = 1 << 0;
pub const DOUBLES_CONSONANT: u16      = 1 << 1;
pub const TAKES_THIN_SUFFIX: u16      = 1 << 2;
pub const RESISTS_SOFTENING: u16      = 1 << 3;
pub const FORCES_SOFTENING: u16       = 1 << 4;
pub const IS_PROPER_NOUN: u16         = 1 << 5;
pub const IRREGULAR_BUFFER: u16       = 1 << 6;
pub const COMPOUND_SUFFIX_DROP: u16   = 1 << 7;
pub const NARROWS_VOWEL_EXC: u16      = 1 << 8;
pub const TAKES_IRREGULAR_AORIST: u16 = 1 << 9;
pub const CHANGES_ROOT_ON_DATIVE: u16 = 1 << 10;
pub const IS_ABBREVIATION: u16        = 1 << 11;
pub const IS_NUMBER: u16              = 1 << 12;
pub const IS_VERB: u16                = 1 << 13; // EKLENDİ: Bu kök bir fiildir. (0010000000000000)
pub const IS_TRANSITIVE: u16          = 1 << 14; // Nesne Alır (Örn: okumak -> kitabı okumak)
pub const IS_INTRANSITIVE: u16        = 1 << 15; // Nesne Almaz (Örn: uyumak -> kitabı

#[derive(Debug, Clone)]
pub struct RootNode {
    pub children: BTreeMap<char, RootNode>,
    pub is_end_of_word: bool,
    pub flags: u16,
    pub domain: String, // YENİ EKLENDİ
}

impl RootNode {
    pub fn new() -> Self {
        RootNode { children: BTreeMap::new(), is_end_of_word: false, flags: 0, domain: "GENERAL".to_string() }
    }
}

// =========================================================================
// OMEGA NOKTASI - BULANIK KÖK SONUCU (FUZZY ROOT RESULT)
// =========================================================================
#[derive(Debug, Clone)]
pub struct FuzzyRootResult {
    pub root_word: String,     // Bulunan kusursuz kök (Örn: "göz")
    pub dna: u16,              // 16-bit DNA
    pub penalty: f32,          // Kök için kesilen ceza puanı
    pub consumed_len: usize,   // Ham girdiden kaç harf kestiği (Örn: "gızlük" için 3)
    pub domain: String,
}

#[derive(Debug, Clone)]
pub struct RootTrie {
    root: RootNode,
}

impl RootTrie {
    pub fn new() -> Self {
        RootTrie { root: RootNode::new() }
    }

    pub fn get_domain_fast(&self, word: &str) -> Option<String> {
        let mut current_node = &self.root;
        let mut last_valid_domain = "GENERAL".to_string();

        // OMEGA NOKTASI - PREFIX (ÖN EK) DOMAİN TARAMASI
        // "saatinda" kelimesi ağaca girer. 's', 'a', 'a', 't' düğümlerinden geçer.
        // 't' düğümünde is_end_of_word = true ve domain = "TIME" görür. Bunu hafızaya alır!
        // Sonra 'i' harfini bulamayıp kopsa bile, hafızasındaki "TIME" bilgisini geri döndürür!
        for ch in word.chars() {
            match current_node.children.get(&ch) {
                Some(node) => {
                    current_node = node;
                    if current_node.is_end_of_word && current_node.domain != "GENERAL" {
                        last_valid_domain = current_node.domain.clone();
                    }
                },
                None => break,
            }
        }
        Some(last_valid_domain)
    }

    pub fn insert(&mut self, word: &str, dna_flags: u16, domain: &str) { // YENİ: domain parametresi eklendi
        let mut current_node = &mut self.root;
        for ch in word.chars() {
            current_node = current_node.children.entry(ch).or_insert_with(RootNode::new);
        }
        current_node.is_end_of_word = true;
        current_node.flags = dna_flags;
        current_node.domain = domain.to_string(); // YENİ
    }

    pub fn search_exact(&self, word: &str) -> Option<u16> {
        let mut current_node = &self.root;
        for ch in word.chars() {
            match current_node.children.get(&ch) {
                Some(node) => current_node = node,
                None => return None,
            }
        }
        if current_node.is_end_of_word { Some(current_node.flags) } else { None }
    }

    // =========================================================================
    // OMEGA NOKTASI - LEVENSHTEIN AUTOMATON (BULANIK TARAMA)
    // =========================================================================
    pub fn search_fuzzy(&self, token: &str, max_penalty: f32) -> Vec<FuzzyRootResult> {
        let token_chars: Vec<char> = token.chars().collect();
        let mut results = Vec::new();
        
        // DP Matrisinin ilk satırı (Kullanıcının girdiği kelime uzunluğu kadar)
        let mut initial_row = vec![0.0; token_chars.len() + 1];
        for i in 0..=token_chars.len() {
            initial_row[i] = i as f32; // Başlangıç silme maliyetleri
        }

        for (&ch, child_node) in &self.root.children {
            let mut path = String::new();
            path.push(ch);
            self.dfs_dp_automaton(
                child_node, 
                &token_chars, 
                path, 
                &initial_row, 
                max_penalty, 
                &mut results
            );
        }

        results.sort_by(|a, b| a.penalty.partial_cmp(&b.penalty).unwrap());
        results
    }

    fn dfs_dp_automaton(
        &self,
        node: &RootNode,
        token_chars: &[char],
        current_path: String,
        prev_row: &[f32],
        max_penalty: f32,
        results: &mut Vec<FuzzyRootResult>
    ) {
        let m = token_chars.len();
        
        // ==========================================
        // OMEGA NOKTASI - ZERO-ALLOCATION (SIFIR HEAP KOPYASI)
        // ==========================================
        // Vektör klonlamak yerine Stack (Yığın) üzerinde sabit 32 boyutlu dizi açıyoruz.
        // Bir kelime en fazla 31 harf olabilir. O(V) bellek maliyeti O(1)'e düştü!
        let mut current_row = [0.0; 32]; 
        current_row[0] = prev_row[0] + 1.0; 

        let mut min_penalty_in_row = current_row[0];
        let current_char = current_path.chars().last().unwrap();

        for i in 1..=m {
            let insert_cost = current_row[i - 1] + 1.0; 
            let delete_cost = prev_row[i] + 1.0;        
            
            let sub_cost = prev_row[i - 1] + if token_chars[i - 1] == current_char { 
                0.0 
            } else { 
                char_substitution_cost(current_char, token_chars[i - 1]) 
            };
            
            current_row[i] = f32::min(insert_cost, f32::min(delete_cost, sub_cost));
            
            if current_row[i] < min_penalty_in_row {
                min_penalty_in_row = current_row[i];
            }
        }

        if min_penalty_in_row > max_penalty { return; }

        if node.is_end_of_word {
            let mut best_penalty = 1000.0;
            let mut consumed_len = 0;
            
            for i in (0..=m).rev() {
                if current_row[i] <= max_penalty && current_row[i] < best_penalty {
                    best_penalty = current_row[i];
                    consumed_len = i;
                }
            }

            if best_penalty <= max_penalty {
                results.push(FuzzyRootResult {
                    root_word: current_path.clone(),
                    dna: node.flags,
                    penalty: best_penalty,
                    consumed_len,
                    domain: node.domain.clone(),
                });
            }
        }

        for (&ch, child_node) in &node.children {
            let mut new_path = current_path.clone();
            new_path.push(ch);
            self.dfs_dp_automaton(
                child_node,
                token_chars,
                new_path,
                &current_row[..m + 1], // Slice olarak fonksiyona geçirilir
                max_penalty,
                results
            );
        }
    }
}