#![allow(dead_code)]

use crate::core::morph_engine::MorphEngine;
use crate::core::tokenizer::{TokenObject, AnchorStatus};

// BÖLÜM 2: Kapsam Analizi ve Yalıtım
// Şimdilik sadece D0 (Ana Evren) boyutunda, güvenli boşluk parçalaması yapıyoruz.
// Gelecekte N-Gram uzay hatalarını (arabamıngeldiğini -> araba_mın_geldiğini) çözen makası da buraya ekleyeceğiz.

pub struct OmegaCompiler<'a> {
    pub engine: &'a MorphEngine,
}

impl<'a> OmegaCompiler<'a> {
    pub fn new(engine: &'a MorphEngine) -> Self {
        OmegaCompiler { engine }
    }

    /// Ham cümleyi alır, atomlarına böler, onarır ve birleştirir.
    pub fn compile_sentence(&self, raw_sentence: &str) -> String {
        println!("==================================================");
        println!("OMEGA COMPILER - D0 EVRENİ (TAM KAPASİTE ANALİZ)");
        println!("GİRDİ: [{}]", raw_sentence);
        println!("==================================================\n");

        // BÖLÜM 1: KISALTMA SÖZLÜĞÜ (L_abbr) YÜKLEMESİ
        let mut l_abbr = std::collections::HashMap::new();
        l_abbr.insert("prof", "profesör");
        l_abbr.insert("dr", "doktor");
        l_abbr.insert("av", "avukat");
        l_abbr.insert("tc", "türkiye"); // T.C. gibi durumlar için

        let mut n_in_total = 0;
        let mut n_decay_total = 0; // YENİ: Sönümlenen işaretler
        let n_gen_total = 0;   // YENİ: Gelecekte üretilecek hayalet işaretler
        let mut n_out_total = 0;

        let raw_tokens: Vec<&str> = raw_sentence.split_whitespace().collect();

        // 1. ÖN TARAMA (Pre-scan for Semantic Resonance)
        let mut active_domains = Vec::new();
        for word in &raw_tokens {
            let clean = word.to_lowercase().replace(&['.', ',', '\'', '!', '?'][..], "");
            // Eğer temizlenmiş kelime Trie'de kayıpsız (0 ceza ile) bulunuyorsa ve bir domain'i varsa:
            if let Some(domain) = self.engine.root_dict.get_domain_fast(&clean) {
                if domain != "GENERAL" && !active_domains.contains(&domain) {
                    active_domains.push(domain);
                }
            }
        }
        
        if !active_domains.is_empty() {
            println!("  [KAHİN] Cümlenin Semantik Enerjisi Tespit Edildi: {:?}", active_domains);
        }

        let mut final_tokens = Vec::new();

        let mut i = 0;
        while i < raw_tokens.len() {
            let raw_word = raw_tokens[i];

            // ==========================================
            // OMEGA NOKTASI - KAPSAM KALKANI (ZERO-GRAVITY ZONE)
            // Eğer bu kelime yalıtılmış bir alt evren ise, Motoru durdur ve atla!
            // ==========================================
            if raw_word.contains("__SCOPE_") {
                println!("  [YALITIM] {} yalıtılmış evren tespit edildi, dokunulmadan geçiliyor...", raw_word);
                final_tokens.push(raw_word.to_string());
                i += 1;
                continue;
            }

            let mut active_token = TokenObject::new(raw_word);
            
            println!("--- Analiz: [{}] ---", raw_word);
            n_in_total += active_token.get_n_in();

            // BÖLÜM 4: GENİŞLEME VE SÖNÜMLENME (Mutation & Decay)
            let mut is_expanded = false;
            let expected_decay_punct = '.'; // Genelde kısaltmalar noktayla biter
            
            // Kullanıcının girdiği steril kelimenin ("profun") bizim sözlükteki kısaltmalarla ("prof") başlayıp başlamadığını kontrol et!
            for (abbr, expanded) in l_abbr.iter() {
                if active_token.normalized_text.starts_with(abbr) {
                    
                    // "profun" içinden "prof" u kes, geriye "un" kalsın.
                    let suffix_part = &active_token.normalized_text[abbr.len()..]; 
                    
                    println!("  [GENİŞLEME] Kısaltma tespit edildi: '{}' -> '{}' + Ek: '{}'", abbr, expanded, suffix_part);
                    
                    // Yeni steril form: "profesör" + "un" = "profesörun"
                    active_token.normalized_text = format!("{}{}", expanded, suffix_part);
                    is_expanded = true;

                    // Noktayı söndürme (Decay) işlemi: Kullanıcı nokta KOYDUYSA söndür, koymadıysa (sokak stili) boşver.
                    if active_token.decay_punctuation(expected_decay_punct) {
                        n_decay_total += 1;
                        println!("  [SÖNÜMLENME] Kısaltma noktası imha edildi! N_decay: {}", n_decay_total);
                    } else {
                        // Eğer nokta yoksa ama kesme işareti varsa (Prof'un) onu da söndürebiliriz, çünkü "Profesör'ün" derken rekonstrüksiyon kesmeyi kendi de geri koyar veya koymaz (T_final formatına göre). 
                        // Şimdilik sadece noktayı söndürüyoruz.
                    }
                    
                    break; // Bir kısaltma bulduk, döngüden çık
                }
            }

            let mut best_stem = String::new();
            let mut best_penalty = 1000.0;
            let mut consumed_tokens = 1;

            // 1. TEKİL SİMÜLASYON
            let _p1 = match self.engine.parse_with_correction(&active_token.normalized_text, &active_domains) {
                Ok((stem, penalty)) => {
                    best_stem = stem;
                    best_penalty = penalty;
                    penalty
                },
                Err(_) => 1000.0,
            };

            // 2. N-GRAM LATTICE SİMÜLASYONU (İleri Bakış)
            if !is_expanded && i + 1 < raw_tokens.len() {
                let next_word = raw_tokens[i+1];
                
                // ==========================================
                // OMEGA NOKTASI - KARADELİK KALKANI
                // N-Gram motoru, yalıtılmış alt evrenlere veya sentetik çapalara temas edemez!
                // ==========================================
                if !next_word.contains("__SCOPE_") {
                    let token_next = TokenObject::new(next_word);
                    
                    // DİKKAT: Verilen kodda eski metot imzaları vardı. Bozmamak adına mevcut koddaki "active_domains" parametresi ve (String, f32) dönüşüyle uyumlu kılındı.
                    let p2 = match self.engine.parse_with_correction(&token_next.normalized_text, &active_domains) {
                        Ok((_, penalty)) => penalty,
                        Err(_) => 1000.0,
                    };

                    let separated_penalty = if best_penalty == 1000.0 && p2 == 1000.0 { 2000.0 } else { best_penalty + p2 };

                    let token_merged = active_token.merge(&token_next);
                    
                    if let Ok((m_stem, m_penalty)) = self.engine.parse_with_correction(&token_merged.normalized_text, &active_domains) {
                        
                        let mut right_merge_is_better = false;
                        if i + 2 < raw_tokens.len() {
                            let next_next_word = raw_tokens[i+2];
                            
                            // Sağdaki kelime de bir Kapsam Çapası değilse birleştirme testi yap!
                            if !next_next_word.contains("__SCOPE_") {
                                let token_next_next = TokenObject::new(next_next_word);
                                let token_right = token_next.merge(&token_next_next);
                                
                                if let Ok((_, r_penalty)) = self.engine.parse_with_correction(&token_right.normalized_text, &active_domains) {
                                    if r_penalty < m_penalty { right_merge_is_better = true; }
                                }
                            }
                        }

                        if m_penalty < separated_penalty && !right_merge_is_better {
                            println!("  [KUANTUM SIÇRAMASI] Boşluk hatası onarıldı! (Ceza: {})", m_penalty);
                            best_stem = m_stem;
                            best_penalty = m_penalty;
                            consumed_tokens = 2; 
                            active_token = token_merged;
                            
                            // İkinci kelimeden gelen virgülleri/noktaları da Giren İşaretlere ekle!
                            n_in_total += token_next.get_n_in();
                        }
                    }
                }
            }

            // 3. KARAR VE REKONSTRÜKSİYON
            if best_penalty < 1000.0 {
                match active_token.reconstruct(&best_stem) {
                    Ok(reconstructed_word) => {
                        n_out_total += active_token.anchors.iter().filter(|a| a.status == AnchorStatus::Active).count();
                        final_tokens.push(reconstructed_word.clone());
                        println!("  [BAŞARI] Çıktı -> '{}'", reconstructed_word);
                    },
                    Err(e) => {
                        println!("  [KRİTİK HATA] Rekonstrüksiyon: {}", e);
                        n_out_total += active_token.anchors.iter().filter(|a| a.status != AnchorStatus::Decayed).count();
                        final_tokens.push(raw_word.to_string());
                    }
                }
            } else {
                println!("  [İPTAL] Motor bu bloğu çözemedi. Orijinal hali korunuyor.");
                n_out_total += active_token.anchors.iter().filter(|a| a.status != AnchorStatus::Decayed).count();
                final_tokens.push(raw_word.to_string());
            }
            
            println!("-----------------------------");
            i += consumed_tokens;
        }

        println!("\n=== BÖLÜM 5: TOPOLOJİK DOĞRULAMA ===");
        println!("Denklem: N_in({}) + N_gen({}) - N_decay({}) = N_out({})", n_in_total, n_gen_total, n_decay_total, n_out_total);
        if n_in_total + n_gen_total - n_decay_total == n_out_total {
            println!("[KUSURSUZ] Mutlak korunum sağlandı.");
        } else {
            println!("[İHLAL] Korunum yasası kırıldı!");
        }

        let final_sentence = final_tokens.join(" ");
        println!("\n[ T_final (MUTLAK ÇIKTI) ]: {}\n", final_sentence);
        
        final_sentence
    }

    // BÖLÜM 2 VE BÖLÜM 6: HİYERARŞİK KAPSAM YALITIMI VE REKONSTRÜKSİYONU
    pub fn compile_multiverse(&self, raw_text: &str) -> String {
        println!("==================================================");
        println!("OMEGA COMPILER - ÇOKLU EVREN (MULTIVERSE) MİMARİSİ");
        println!("GİRDİ: [{}]", raw_text);
        println!("==================================================\n");

        let mut scopes = Vec::new();
        let mut d0_text = String::new();
        let mut chars = raw_text.chars().peekable();
        
        let mut in_quote = false;
        let mut in_paren = false;
        let mut current_scope = String::new();
        let mut scope_char = ' ';

        // 1. BOYUTLARI YIRTMA (Extraction)
        while let Some(c) = chars.next() {
            if !in_quote && !in_paren {
                if c == '"' {
                    in_quote = true;
                    scope_char = '"';
                    d0_text.push_str(&format!("__SCOPE_{}__", scopes.len()));
                } else if c == '(' {
                    in_paren = true;
                    scope_char = '(';
                    d0_text.push_str(&format!("__SCOPE_{}__", scopes.len()));
                } else {
                    d0_text.push(c);
                }
            } else {
                // Kapsamın içindeyiz
                if (in_quote && c == '"') || (in_paren && c == ')') {
                    scopes.push((scope_char, current_scope.clone()));
                    current_scope.clear();
                    in_quote = false;
                    in_paren = false;
                } else {
                    current_scope.push(c);
                }
            }
        }

        println!("[BÖLÜM 2] Evrenler Yalıtıldı. Ana Uzay (D0): [{}]", d0_text.trim());

        // 2. ANA EVRENİ (D0) DERLE
        let mut compiled_d0 = self.compile_sentence(&d0_text);

        // 3. ALT EVRENLERİ (D1, D2) DERLE VE ANA UZAYA GERİ YERLEŞTİR (BÖLÜM 6)
        println!("\n=== BÖLÜM 6: TOPOLOJİK REKONSTRÜKSİYON VE YERLEŞTİRME ===");
        for (i, (boundary, raw_scope)) in scopes.iter().enumerate() {
            println!("\n>>> ALT EVREN (D{}) LABORATUVARA ALINIYOR: [{}] <<<", i + 1, raw_scope);
            
            // Alt evreni bağımsız olarak derle!
            let compiled_scope = self.compile_sentence(raw_scope);
            
            let marker = format!("__SCOPE_{}__", i);
            let replacement = if *boundary == '"' {
                format!("\"{}\"", compiled_scope) // Tırnakları geri koy
            } else {
                format!("({})", compiled_scope)   // Parantezleri geri koy
            };
            
            // D0 evrenindeki sentetik çapayı, onarılmış gerçek evrenle değiştir
            compiled_d0 = compiled_d0.replace(&marker, &replacement);
        }

        // Fazladan oluşan boşlukları (spacing) standardize et
        let final_text = compiled_d0.split_whitespace().collect::<Vec<&str>>().join(" ");
        
        println!("\n[ T_final (MUTLAK MULTIVERSE ÇIKTISI) ]: {}\n", final_text);
        final_text
    }
}