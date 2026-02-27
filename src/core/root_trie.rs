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
    /// Trie ağacında hata toleranslı derinlemesine arama yapar.
    pub fn search_fuzzy(&self, token: &str, max_penalty: f32) -> Vec<FuzzyRootResult> {
        let mut results = Vec::new();
        let chars: Vec<char> = token.chars().collect();
        
        self.dfs_fuzzy(&self.root, &chars, 0, String::new(), 0.0, max_penalty, &mut results);
        
        // En düşük ceza puanlıları öne al
        results.sort_by(|a, b| a.penalty.partial_cmp(&b.penalty).unwrap());
        results
    }

fn dfs_fuzzy(
        &self,
        node: &RootNode,
        token_chars: &[char],
        char_idx: usize,
        current_path: String,
        current_penalty: f32,
        max_penalty: f32,
        results: &mut Vec<FuzzyRootResult>
    ) {
        // ZİRVE: Geçerli bir kök bulduk!
        if node.is_end_of_word {
            // DİNAMİK TOLERANS: Orijinal kökün uzunluğuna göre ceza sınırı. 
            // Kullanıcı harf yutsa bile gerçek kök uzunluğuna göre esneklik sağlar.
            let max_allowed_for_this_len = (current_path.chars().count() as f32) * 0.45;
            
            if current_penalty <= max_allowed_for_this_len && current_penalty <= max_penalty {
                results.push(FuzzyRootResult {
                    root_word: current_path.clone(),
                    dna: node.flags,
                    penalty: current_penalty,
                    consumed_len: char_idx, // Kullanıcı metninden tam olarak kaç harf emdiğimizi bildirir!
                    domain: "GENERAL".to_string(),
                });
            }
        }

        // Eğer ceza limitini aştıysak bu paralel evreni anında yok et (Pruning)
        if current_penalty > max_penalty {
            return;
        }

        // 1. ZAMAN ATLAMASI (Insertion - Kullanıcı klavyede fazladan harfe basmış)
        // Kullanıcının metninde 1 harf ilerliyoruz ama Trie Ağacında (Kök Kütüphanesinde) ilerlemiyoruz!
        if char_idx < token_chars.len() {
            let ins_cost = 1.0; // Fazla harf cezası
            if current_penalty + ins_cost <= max_penalty {
                self.dfs_fuzzy(
                    node,
                    token_chars,
                    char_idx + 1, // Kullanıcı metni atlandı
                    current_path.clone(),
                    current_penalty + ins_cost,
                    max_penalty,
                    results
                );
            }
        }

        for (&child_char, child_node) in &node.children {
            // 2. GÖVDE ÇATALLANMASI (Deletion - Kullanıcı harf yutmuş, eksik yazmış)
            // Ağaçta (Kütüphanede) ilerliyoruz, ama kullanıcının metninde yerimizde sayıyoruz!
            let del_cost = 1.0; // Eksik harf cezası
            if current_penalty + del_cost <= max_penalty {
                let mut new_path = current_path.clone();
                new_path.push(child_char);
                self.dfs_fuzzy(
                    child_node,
                    token_chars,
                    char_idx, // char_idx ARTMIYOR, kullanıcı metninde bekliyoruz
                    new_path,
                    current_penalty + del_cost,
                    max_penalty,
                    results
                );
            }

            if char_idx < token_chars.len() {
                let target_char = token_chars[char_idx];
                
                // 3. YERİNE KOYMA VEYA KUSURSUZ EŞLEŞME (Substitution / Match)
                let sub_cost = char_substitution_cost(target_char, child_char);
                if current_penalty + sub_cost <= max_penalty {
                    let mut new_path = current_path.clone();
                    new_path.push(child_char);
                    self.dfs_fuzzy(
                        child_node,
                        token_chars,
                        char_idx + 1,
                        new_path,
                        current_penalty + sub_cost,
                        max_penalty,
                        results
                    );
                }

                // 4. TRANSPOZİSYON (Swapping - Harflerin yeri değişmiş, örn: gzö -> göz)
                if char_idx + 1 < token_chars.len() {
                    let next_target_char = token_chars[char_idx + 1];
                    // Trie'de şu anki harf = kullanıcının bir sonraki harfi VE Trie'nin bir sonraki harfi = kullanıcının şu anki harfi
                    if child_char == next_target_char {
                        if let Some(grandchild_node) = child_node.children.get(&target_char) {
                            let swap_cost = 0.5; // El sürçmesi, affedilebilir bir hata!
                            if current_penalty + swap_cost <= max_penalty {
                                let mut new_path = current_path.clone();
                                new_path.push(child_char);
                                new_path.push(target_char);
                                self.dfs_fuzzy(
                                    grandchild_node,
                                    token_chars,
                                    char_idx + 2, // 2 harf birden emdik
                                    new_path,
                                    current_penalty + swap_cost,
                                    max_penalty,
                                    results
                                );
                            }
                        }
                    }
                }
            }
        }
    }}