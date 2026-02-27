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

#[derive(Debug, Clone)]
pub struct RootNode {
    pub children: BTreeMap<char, RootNode>,
    pub is_end_of_word: bool,
    pub flags: u16,
}

impl RootNode {
    pub fn new() -> Self {
        RootNode { children: BTreeMap::new(), is_end_of_word: false, flags: 0 }
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

    pub fn get_domain_fast(&self, _word: &str) -> Option<String> {
        // İleride ağaç düğümlerine (RootNode) domain eklediğimizde burası gerçek veriyi çekecek.
        // Şimdilik sistemin çökmemesi ve Kahin'in nötr kalması için "GENERAL" dönüyoruz.
        Some("GENERAL".to_string())
    }

    pub fn insert(&mut self, word: &str, dna_flags: u16) {
        let mut current_node = &mut self.root;
        for ch in word.chars() {
            current_node = current_node.children.entry(ch).or_insert_with(RootNode::new);
        }
        current_node.is_end_of_word = true;
        current_node.flags = dna_flags;
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
        let mut current_row = vec![0.0; m + 1];
        current_row[0] = prev_row[0] + 1.0; // Ağaçta ilerledikçe kullanıcı metninde yerinde sayma (Insertion to Tree)

        let mut min_penalty_in_row = current_row[0];
        let current_char = current_path.chars().last().unwrap();

        for i in 1..=m {
            let insert_cost = current_row[i - 1] + 1.0; // Kullanıcı fazladan harf basmış
            let delete_cost = prev_row[i] + 1.0;        // Kullanıcı harf yutmuş
            
            // Yer değiştirme veya eşleşme maliyeti
            let sub_cost = prev_row[i - 1] + if token_chars[i - 1] == current_char { 
                0.0 
            } else { 
                crate::core::distance::char_substitution_cost(token_chars[i - 1], current_char) 
            };
            
            current_row[i] = f32::min(insert_cost, f32::min(delete_cost, sub_cost));
            
            if current_row[i] < min_penalty_in_row {
                min_penalty_in_row = current_row[i];
            }
        }

        // AGRESİF BUDAMA (Pruning): Eğer bu satırdaki en düşük ceza bile sınırı aştıysa, 
        // bu dalın devamına (alt trilyonlarca ihtimale) bakma! Sonsuz evrenleri yok et!
        if min_penalty_in_row > max_penalty {
            return;
        }

        // Hedef Bulundu: Eğer ağaçta bir kelime bittiyse ve kullanıcının metniyle eşleşme cezası sınırın altındaysa
        if node.is_end_of_word {
            // Kullanıcının metninden kaç harf emdiğimizi bul (en düşük cezalı sütun)
            let mut best_penalty = 1000.0;
            let mut consumed_len = 0;
            
            // Sadece kelimenin anlamlı bir kısmını tüketmişse kabul et (Örn: en az %50'sini)
            for i in 0..=m {
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
                    domain: "GENERAL".to_string(), // Derleme hatası olmaması için eklendi.
                });
            }
        }

        // Ağacın derinliklerine inmeye devam et
        for (&ch, child_node) in &node.children {
            let mut new_path = current_path.clone();
            new_path.push(ch);
            self.dfs_dp_automaton(
                child_node,
                token_chars,
                new_path,
                &current_row, // Kuantum durumu bir sonraki nesle aktarılır
                max_penalty,
                results
            );
        }
    }
}