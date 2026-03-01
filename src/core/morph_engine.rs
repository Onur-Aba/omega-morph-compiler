#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_assignments)]

use std::collections::HashMap;
use crate::core::root_trie::{RootTrie, IS_VERB, RESISTS_SOFTENING};
use crate::core::suffix_fsm::{MorphState, SuffixNode};
use crate::core::phonology::synthesize_suffix;

#[derive(Debug, Clone)]
pub struct PathResult {
    pub final_stem: String,
    pub total_penalty: f32,
    pub final_state: MorphState, // YENİ: Kelimenin bittiği durak
}

pub struct MorphEngine {
    pub root_dict: RootTrie,
    pub suffix_graph: HashMap<MorphState, Vec<SuffixNode>>,
    pub domain_matrix: HashMap<String, Vec<String>>, // YENİ: Semantik Kardeşlik Matrisi
}

impl MorphEngine {
    pub fn new(root_dict: RootTrie) -> Self {
        MorphEngine { 
            root_dict, 
            suffix_graph: HashMap::new(),
            domain_matrix: HashMap::new(), // YENİ: Başlangıçta boş atıyoruz
        }
    }

    pub fn register_suffix_route(&mut self, from_state: MorphState, suffix: SuffixNode) {
        self.suffix_graph.entry(from_state).or_insert_with(Vec::new).push(suffix);
    }

    /// Tam Otonom Kuantum Tarayıcı (Kökü kendi bulur, cezayı biriktirir)
    pub fn parse_with_correction(
        &self, 
        token: &str, 
        active_domains: &HashMap<String, f32> // O an cümlede aktif olan semantik enerjiler
    ) -> Result<(String, f32, MorphState), String> { // DÖNÜŞ TİPİ GÜNCELLENDİ
        println!("--- Tam Otonom Toleranslı Tarama Başlıyor: [{}] ---", token);
        
        // 1. Olası tüm kökleri bulanık tarama (Fuzzy Search) ile ağaçtan kopar 
        // Tolerans 1.3 yapıldı ki 'islem' ararken 'işlem' kökü (s->ş farkı) budanıp yok olmasın!
        let possible_roots = self.root_dict.search_fuzzy(token, 1.3);
        
        if possible_roots.is_empty() {
            return Err(format!("[HATA] '{}' için hiçbir mantıklı kök bulunamadı.", token));
        }

        let mut valid_paths = Vec::new();

        // Kuantum Belleği (Aynı State ve aynı Remaining Text için en düşük cezayı tutar)
        // Eğer bu noktaya daha önce daha iyi bir skorla geldiysek, dallanmayı durdurur. O(B^D) -> O(V+E)
        let mut memo: HashMap<(MorphState, usize), f32> = HashMap::new();

        // 2. Bulunan her kök adayı için FSM trenini çalıştır
        for root_candidate in possible_roots {
            // ==========================================
            // OMEGA NOKTASI - SAHTE YUMUŞAMA KALKANI (ANTI-SOFTENING)
            // ==========================================
            // Kullanıcı "hukuga" yazdı, kök "hukuk" bulundu.
            // Kök yumuşamayı reddediyorsa (RESISTS_SOFTENING), kalan kısımdaki (Örn: "ga") 
            // ilk harfin o sahte yumuşama olup olmadığını kontrol et ve düzelt!
            let mut current_token = token.to_string();
            
            if (root_candidate.dna & crate::core::root_trie::RESISTS_SOFTENING) != 0 {
                if let Some(last_root_char) = root_candidate.root_word.chars().last() {
                    if ['p', 'ç', 't', 'k'].contains(&last_root_char) {
                        let consumed = root_candidate.consumed_len;
                        let token_chars: Vec<char> = token.chars().collect();
                        
                        // Kullanıcının yazdığı kelimede kökün bittiği yerdeki harfe bak
                        if consumed > 0 && consumed <= token_chars.len() {
                            let user_char = token_chars[consumed - 1];
                            
                            // Kullanıcı k yerine g/ğ, t yerine d yazmışsa ve bu kök yumuşamıyorsa!
                            if (last_root_char == 'k' && (user_char == 'ğ' || user_char == 'g')) ||
                               (last_root_char == 't' && user_char == 'd') ||
                               (last_root_char == 'p' && user_char == 'b') ||
                               (last_root_char == 'ç' && user_char == 'c') {
                                
                                // Kelimeyi kullanıcının yazdığı o bozuk harften kurtar
                                // (Örn: "hukuga" kelimesini "hukuka" yap ki FSM kalan "a" ekini net görsün!)
                                let mut fixed_chars = token_chars.clone();
                                fixed_chars[consumed - 1] = last_root_char;
                                current_token = fixed_chars.into_iter().collect();
                            }
                        }
                    }
                }
            }

            // UTF-8 Güvenli Kesim (Artık düzeltilmiş current_token üzerinden kalan eki buluruz)
            let char_indices: Vec<(usize, char)> = current_token.char_indices().collect();
            let remaining = if root_candidate.consumed_len < char_indices.len() {
                let split_byte_index = char_indices[root_candidate.consumed_len].0;
                &current_token[split_byte_index..]
            } else {
                "" // Kök kelimenin tamamını tükettiyse ek kalmamıştır
            };

            // ==========================================
            // OMEGA NOKTASI - SEMANTİK REZONANS (MUTLAK KÖK KORUMASI)
            // ==========================================
            let mut context_penalty = root_candidate.penalty;

            if root_candidate.domain == "GENERAL" {
                // FİİLLER: Nötr.
            } else if let Some(&bonus) = active_domains.get(&root_candidate.domain) {
                let root_len = root_candidate.root_word.chars().count();
                let remaining_len = remaining.chars().count();
                
                // MİMARİ KURAL 1: Kök kısacık (1-3 harf) ve ek kısmı kökten daha uzunsa bu bir halüsinasyondur!
                // "aykiri" kelimesinden "ay" çıkarıp geriye koca bir "kiri" bırakmasın.
                let safe_bonus = if root_len <= 3 && remaining_len > root_len {
                    0.0 // Kısa köklere hiç ödül verme ki "aykiri"yi "ay" zannetmesin!
                } else {
                    bonus
                };
                
                context_penalty -= safe_bonus;
            }

            // MİMARİ KURAL 2: Orijinal kelime ile (Örn: "aykiri") kök (Örn: "ay") arasındaki HARF FARKI 
            // ne kadar büyükse (yani "remaining" ne kadar uzunsa), kök o kadar "Uydurmadır".
            // Kök dışı kalan her harf için sisteme GİZLİ CEZA BİNDİR! 
            // "aykiri" kökünde remaining = "", ceza: 0.0
            // "ay" kökünde remaining = "kiri", ceza: 4 harf * 0.5 = +2.0! (Böylece "ay" kökü asla seçilemez)
            let remaining_char_count = remaining.chars().count();
            if remaining_char_count > 0 {
                context_penalty += (remaining_char_count as f32) * 0.4;
            }

            // MİMARİ KURAL 3: Başlangıç cezası hiçbir şekilde negatif olamaz.
            if context_penalty < 0.0 {
                context_penalty = 0.0;
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
                &root_candidate.root_word, // Kökün kendisini referans olarak gönderiyoruz
                root_candidate.dna,
                context_penalty, // Kökten gelen ceza puanını aktar
                &mut valid_paths,
                &mut memo
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

        Ok((best_path.final_stem.clone(), best_path.total_penalty, best_path.final_state.clone()))
    }

    fn traverse_fsm_viterbi(&self, current_state: MorphState, current_stem: &str, remaining: &str, root_word: &str, root_dna: u16, current_penalty: f32, valid_paths: &mut Vec<PathResult>, memo: &mut HashMap<(MorphState, usize), f32>) {
        
        // ==========================================
        // DİNAMİK PROGRAMLAMA: BUDAMA (PRUNING)
        // ==========================================
        let cache_key = (current_state.clone(), remaining.len());
        if let Some(&best_past_penalty) = memo.get(&cache_key) {
            // Eğer bu durağa daha önce daha düşük veya eşit bir cezayla geldiysek, bu paralel evreni YOK ET!
            if current_penalty >= best_past_penalty { return; }
        }
        memo.insert(cache_key, current_penalty);

        if remaining.is_empty() {
            // TÜRKÇE MUTLAK BİTİŞ KURALLARI:
            // Kelime hangi ekleri aldıktan sonra yasal olarak cümlede durabilir?
            let is_valid_termination = matches!(
                current_state,
                MorphState::RootNoun | MorphState::RootVerb |
                MorphState::Case | MorphState::Person | MorphState::Copula |
                MorphState::Tense | MorphState::Question | MorphState::EndOfWord |
                MorphState::Possessive | MorphState::Plural // İsimler çoğul veya iyelikle de bitebilir
            );
                // if current_stem.contains("ora") || current_stem.contains("götür") {
                // println!("  [X-RAY BİTİŞ] Kelime: '{}' | Durak: {:?} | Kabul Edildi mi: {}", 
                //     current_stem, current_state, is_valid_termination);
                // }
            if is_valid_termination {
                valid_paths.push(PathResult {
                    final_stem: current_stem.to_string(),
                    total_penalty: current_penalty,
                    final_state: current_state.clone(), // YENİ
                });
            } else {
                // SESSİZ İPTAL: Kelime bitti ama FSM havada kaldı (Örn: Sadece Olumsuzluk eki alıp bitmiş 'gitme-').
                // Bu yol çöpe atılır.
            }
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
                
                let last_char = current_stem.chars().last().unwrap_or(' ');
                let _stem_ends_with_vowel = vowels.contains(&last_char); // Sarı uyarı gitmesi için _ eklendi

                // BUFFER S, N, Y Mantığı burada phonology'ye devredilmişti, o yüzden burası boş.

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
                    
                    // X-RAY 1: Hangi ek nasıl sentezlendi? (Gerektiğinde açabilirsin)
                    // if actual_stem_output == "ora" || actual_stem_output.starts_with("götür") {
                    //     println!("  [X-RAY] Gövde: '{}' | Ek: '{}' -> Üretilen: '{}' | Sonuç: '{}'", 
                    //         actual_stem_output, suffix.id, synthesized_for_output, new_stem);
                    // }
                    self.traverse_fsm_viterbi(
                        suffix.output_state.clone(), 
                        &new_stem, 
                        &new_remaining, 
                        root_word,
                        root_dna, 
                        current_penalty + penalty, 
                        valid_paths,
                        memo
                    );
                }
            }
        }
    } 
}