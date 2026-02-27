#![allow(dead_code)]

use std::collections::HashMap;
use crate::core::root_trie::{RootTrie, IS_VERB, RESISTS_SOFTENING};
use crate::core::suffix_fsm::{MorphState, SuffixNode};
use crate::core::phonology::synthesize_suffix;


#[derive(Debug, Clone)]
pub struct PathResult {
    pub final_stem: String,
    pub total_penalty: f32,
}

pub struct MorphEngine {
    pub root_dict: RootTrie,
    pub suffix_graph: HashMap<MorphState, Vec<SuffixNode>>,
}

impl MorphEngine {
    pub fn new(root_dict: RootTrie) -> Self {
        MorphEngine { root_dict, suffix_graph: HashMap::new() }
    }

    pub fn register_suffix_route(&mut self, from_state: MorphState, suffix: SuffixNode) {
        self.suffix_graph.entry(from_state).or_insert_with(Vec::new).push(suffix);
    }

    /// Tam Otonom Kuantum Tarayıcı (Kökü kendi bulur, cezayı biriktirir)
    pub fn parse_with_correction(
        &self, 
        token: &str, 
        active_domains: &[String] // O an cümlede aktif olan semantik enerjiler
    ) -> Result<(String, f32), String> {
        println!("--- Tam Otonom Toleranslı Tarama Başlıyor: [{}] ---", token);
        
        // 1. Olası tüm kökleri bulanık tarama (Fuzzy Search) ile ağaçtan kopar (Maksimum ceza: 1.5)
        let possible_roots = self.root_dict.search_fuzzy(token, 1.5);
        
        if possible_roots.is_empty() {
            return Err(format!("[HATA] '{}' için hiçbir mantıklı kök bulunamadı.", token));
        }

        let mut valid_paths = Vec::new();

        // 2. Bulunan her kök adayı için FSM trenini çalıştır
        for root_candidate in possible_roots {
            // UTF-8 Güvenli Kesim (Kökün tükettiği harf sayısına göre kelimenin kalanını bul)
            let char_indices: Vec<(usize, char)> = token.char_indices().collect();
            
            let remaining = if root_candidate.consumed_len < char_indices.len() {
                let split_byte_index = char_indices[root_candidate.consumed_len].0;
                &token[split_byte_index..]
            } else {
                "" // Kök kelimenin tamamını tükettiyse ek kalmamıştır
            };

            let mut context_penalty = root_candidate.penalty;

            // ==========================================
            // OMEGA NOKTASI - SEMANTİK REZONANS
            // ==========================================
            // Eğer bu kökün ait olduğu domain, cümlenin aktif domainleri içindeyse: BONUS!
            if active_domains.contains(&root_candidate.domain) {
                context_penalty -= 2.0; // Ağır bir bağlam indirimi!
                println!("  [REZONANS] '{}' kelimesi cümlenin '{}' enerjisiyle eşleşti! Bonus uygulandı.", root_candidate.root_word, root_candidate.domain);
            }

            println!("  [KÖK ADAYI DENENİYOR] Kök: '{}' (Ceza: {}) | Kalan: '{}'", 
                root_candidate.root_word, context_penalty, remaining);

            // DNA Analizi: Kök fiil mi, isim mi? Makası ona göre değiştir!
            let start_state = if (root_candidate.dna & IS_VERB) != 0 {
                MorphState::RootVerb
            } else {
                MorphState::RootNoun
            };

            self.traverse_fsm_viterbi(
                start_state,
                &root_candidate.root_word,
                remaining,
                &root_candidate.root_word, // YENİ EKLENDİ (Kökün kendisini referans olarak gönderiyoruz)
                root_candidate.dna,
                context_penalty, // Kökten gelen ceza puanını aktar
                &mut valid_paths
            );
        }

        // 3. Eşzamanlı Kısıt Çözümleme: Tüm yolları ceza puanına göre sırala
        if valid_paths.is_empty() {
            return Err(format!("[HATA] '{}' kelimesi kurtarılamayacak kadar bozuk.", token));
        }

        valid_paths.sort_by(|a, b| a.total_penalty.partial_cmp(&b.total_penalty).unwrap());
        let best_path = &valid_paths[0];

        // LOGLARI TEMİZLEYELİM: Artık çok fazla simülasyon yapılacağı için terminali boğmasın
        // if best_path.total_penalty == 0.0 {
        //     println!("[KUSURSUZ] Kelime kurallara %100 uyuyor: {}", best_path.final_stem);
        // } else {
        //     println!("[ONARILDI] Yazım hatası tespit edildi. Kelime '{}' olarak düzeltildi. (Toplam Ceza: {})", 
        //         best_path.final_stem, best_path.total_penalty);
        // }

        Ok((best_path.final_stem.clone(), best_path.total_penalty))
    }

    fn traverse_fsm_viterbi(&self, current_state: MorphState, current_stem: &str, remaining: &str, root_word: &str, root_dna: u16, current_penalty: f32, valid_paths: &mut Vec<PathResult>) {
        if remaining.is_empty() {
            valid_paths.push(PathResult {
                final_stem: current_stem.to_string(),
                total_penalty: current_penalty,
            });
            return;
        }

        if let Some(suffixes) = self.suffix_graph.get(&current_state) {
            for suffix in suffixes {
                // 1. KULLANICIYI ANLAMA (Puanlama için Sentez)
                // "cAm" -> "cem" veya "cam" üretir.
                let synthesized_for_match = synthesize_suffix(current_stem, root_dna, suffix.flags, &suffix.base_form);
                
                // 2. RESMİ ÇIKTI (Ekrana Basılacak Sentez)
                // Eğer JSON'da "AcAğIm" tanımlıysa onu sentezle, yoksa normal sentezi kullan!
                let synthesized_for_output = if let Some(ref canonical) = suffix.canonical_form {
                    // Makro eki resmi formata çevir ("AcAğIm" -> "eceğim")
                    synthesize_suffix(current_stem, root_dna, suffix.flags, canonical)
                } else {
                    synthesized_for_match.clone()
                };
                
                let mut actual_stem_match = current_stem.to_string();
                let mut actual_stem_output = current_stem.to_string(); // Çıktı için ayrı gövde
                
                let first_char_match = synthesized_for_match.chars().next().unwrap_or(' ');
                let first_char_output = synthesized_for_output.chars().next().unwrap_or(' ');
                let vowels = ['a', 'e', 'ı', 'i', 'o', 'ö', 'u', 'ü', 'A', 'E', 'I', 'İ', 'O', 'Ö', 'U', 'Ü'];
                
                // Ünsüz Yumuşaması Motoru (Hem eşleşme hem çıktı için ayrı ayrı çalışmalı)
                if vowels.contains(&first_char_match) {
                    let mut can_soften = true;
                    if actual_stem_match == root_word && (root_dna & crate::core::root_trie::RESISTS_SOFTENING) != 0 { can_soften = false; }
                    if can_soften { actual_stem_match = crate::core::phonology::apply_consonant_softening(&actual_stem_match); }
                }

                if vowels.contains(&first_char_output) {
                    let mut can_soften = true;
                    if actual_stem_output == root_word && (root_dna & crate::core::root_trie::RESISTS_SOFTENING) != 0 { can_soften = false; }
                    if can_soften { actual_stem_output = crate::core::phonology::apply_consonant_softening(&actual_stem_output); }
                }

                // ==========================================
                // DİNAMİK CEZA VE TÜKETİM (Match formu üzerinden)
                // Sistem "ücem" veya "cem" gördüğünde 0 ceza keser!
                // ==========================================
                let (penalty, consumed_len) = crate::core::distance::match_suffix_fuzzy(&synthesized_for_match, remaining);
                
                if current_penalty + penalty <= 3.0 { 
                    let rem_chars: Vec<char> = remaining.chars().collect();
                    let new_remaining: String = rem_chars.iter().skip(consumed_len).collect();
                    
                    // DİKKAT: Viterbi'nin yeni gövdesi "actual_stem_output + synthesized_for_output" olarak inşa edilir!
                    // Yani "götür" + "eceğim" olur!
                    let new_stem = format!("{}{}", actual_stem_output, synthesized_for_output);
                    
                    self.traverse_fsm_viterbi(
                        suffix.output_state.clone(), 
                        &new_stem, 
                        &new_remaining, 
                        root_word,
                        root_dna, 
                        current_penalty + penalty, 
                        valid_paths
                    );
                }
            }
        }
    }
}