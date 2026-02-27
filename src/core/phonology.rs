#![allow(dead_code)]

use crate::core::root_trie::TAKES_THIN_SUFFIX;
use crate::core::suffix_fsm::{BUFFER_Y, BUFFER_S, BUFFER_N};

// =========================================================================
// OMEGA NOKTASI - FONOLOJİK MUTASYON MOTORU
// =========================================================================

pub fn get_last_vowel(word: &str) -> Option<char> {
    let vowels = ['a', 'e', 'ı', 'i', 'o', 'ö', 'u', 'ü', 'A', 'E', 'I', 'İ', 'O', 'Ö', 'U', 'Ü'];
    word.chars().rev().find(|c| vowels.contains(c))
}

pub fn is_thick_vowel(v: char) -> bool {
    matches!(v, 'a' | 'ı' | 'o' | 'u' | 'A' | 'I' | 'O' | 'U')
}

pub fn resolve_two_way_harmony(root: &str, root_dna: u16, base_suffix: &str) -> String {
    let last_vowel = get_last_vowel(root).unwrap_or('a');
    let mut treat_as_thick = is_thick_vowel(last_vowel);
    if (root_dna & TAKES_THIN_SUFFIX) != 0 { treat_as_thick = false; }
    let resolved_a = if treat_as_thick { 'a' } else { 'e' };
    base_suffix.replace('A', &resolved_a.to_string())
}

pub fn resolve_four_way_harmony(root: &str, root_dna: u16, base_suffix: &str) -> String {
    let last_vowel = get_last_vowel(root).unwrap_or('a');
    let mut resolved_i = match last_vowel {
        'a' | 'ı' | 'A' | 'I' => 'ı',
        'e' | 'i' | 'E' | 'İ' => 'i',
        'o' | 'u' | 'O' | 'U' => 'u',
        'ö' | 'ü' | 'Ö' | 'Ü' => 'ü',
        _ => 'ı',
    };
    if (root_dna & TAKES_THIN_SUFFIX) != 0 && (resolved_i == 'ı' || resolved_i == 'u') {
        resolved_i = if resolved_i == 'ı' { 'i' } else { 'ü' };
    }
    base_suffix.replace('I', &resolved_i.to_string())
}

/// Gelişmiş Sentezleyici: Ünlü Uyumu + Kaynaştırma Harfi (Buffer Letter) Enjeksiyonu
pub fn synthesize_suffix(current_stem: &str, root_dna: u16, suffix_dna: u16, base_suffix: &str) -> String {
    let mut result = if base_suffix.contains('A') {
        resolve_two_way_harmony(current_stem, root_dna, base_suffix)
    } else if base_suffix.contains('I') {
        resolve_four_way_harmony(current_stem, root_dna, base_suffix)
    } else {
        base_suffix.to_string()
    };

    // KAYNAŞTIRMA (BUFFER) DENETİMİ: Kök ünlüyle bitiyorsa ve ek DNA'sı istiyorsa araya harf girer
    let vowels = ['a', 'e', 'ı', 'i', 'o', 'ö', 'u', 'ü'];
    if let Some(last_char) = current_stem.chars().last() {
        if vowels.contains(&last_char) {
            if (suffix_dna & BUFFER_Y) != 0 { result.insert(0, 'y'); }
            else if (suffix_dna & BUFFER_S) != 0 { result.insert(0, 's'); }
            else if (suffix_dna & BUFFER_N) != 0 { result.insert(0, 'n'); }
        }
    }

    result
}
/// Gövdenin sonundaki sert ünsüzleri (p, ç, t, k) yumuşatır.
pub fn apply_consonant_softening(stem: &str) -> String {
    if stem.is_empty() { return stem.to_string(); }
    let mut chars: Vec<char> = stem.chars().collect();
    let last_idx = chars.len() - 1;
    
    match chars[last_idx] {
        'p' => chars[last_idx] = 'b',
        'ç' => chars[last_idx] = 'c',
        't' => chars[last_idx] = 'd',
        'k' => chars[last_idx] = 'ğ', 
        'K' => chars[last_idx] = 'Ğ',
        _ => {}
    }
    chars.into_iter().collect()
}