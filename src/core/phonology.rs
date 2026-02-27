#![allow(dead_code)]

use crate::core::root_trie::TAKES_THIN_SUFFIX;
use crate::core::suffix_fsm::{BUFFER_Y, BUFFER_S, BUFFER_N};

// =========================================================================
// OMEGA NOKTASI - FONOLOJİK MUTASYON MOTORU (KUSURSUZ VERSİYON)
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
    let last_vowel = get_last_vowel(root).unwrap_or('a').to_ascii_lowercase(); // KÜÇÜK HARFE ÇEVİREREK GARANTİLE
    let mut resolved_i = match last_vowel {
        'a' | 'ı' => 'ı',
        'e' | 'i' => 'i',
        'o' | 'u' => 'u',
        'ö' | 'ü' => 'ü',
        _ => 'ı',
    };
    if (root_dna & TAKES_THIN_SUFFIX) != 0 && (resolved_i == 'ı' || resolved_i == 'u') {
        resolved_i = if resolved_i == 'ı' { 'i' } else { 'ü' };
    }
    base_suffix.replace('I', &resolved_i.to_string())
}

/// Gelişmiş Sentezleyici: Ünlü Uyumu + Kaynaştırma Harfi (Buffer Letter) Enjeksiyonu
/// Gelişmiş Sentezleyici: Ünlü Uyumu + Kaynaştırma Harfi (Buffer Letter) Enjeksiyonu
pub fn synthesize_suffix(current_stem: &str, root_dna: u16, suffix_dna: u16, base_suffix: &str) -> String {
    let mut result = String::new();
    let mut current_stem_for_harmony = current_stem.to_string(); // Sentezlendikçe uzayan gövde

    // KAYNAŞTIRMA (BUFFER) DENETİMİ (Sentezin en başında, sadece bir kere yapılır)
    let vowels = ['a', 'e', 'ı', 'i', 'o', 'ö', 'u', 'ü', 'A', 'E', 'I', 'İ', 'O', 'Ö', 'U', 'Ü'];
    if let Some(last_char) = current_stem.chars().last() {
        if vowels.contains(&last_char) {
            if (suffix_dna & BUFFER_Y) != 0 { result.push('y'); current_stem_for_harmony.push('y'); }
            else if (suffix_dna & BUFFER_S) != 0 { result.push('s'); current_stem_for_harmony.push('s'); }
            else if (suffix_dna & BUFFER_N) != 0 { result.push('n'); current_stem_for_harmony.push('n'); }
        }
    }

    // Harf harf işleme ve sürekli güncellenen uyum zinciri (Progressive Harmony)
    for ch in base_suffix.chars() {
        if ch == 'A' {
            let resolved_a = resolve_two_way_harmony(&current_stem_for_harmony, root_dna, "A");
            result.push_str(&resolved_a);
            current_stem_for_harmony.push_str(&resolved_a); // Yeni harf gövdeye eklendi, sonrakileri etkileyecek!
        } else if ch == 'I' || ch == 'İ' {
            let resolved_i = resolve_four_way_harmony(&current_stem_for_harmony, root_dna, "I");
            result.push_str(&resolved_i);
            current_stem_for_harmony.push_str(&resolved_i); // Yeni harf gövdeye eklendi
        } else {
            result.push(ch);
            current_stem_for_harmony.push(ch);
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