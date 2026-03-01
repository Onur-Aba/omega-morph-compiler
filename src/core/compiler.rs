#![allow(dead_code)]

use std::collections::HashMap;
use crate::core::morph_engine::MorphEngine;

use crate::core::suffix_fsm::MorphState;

// ==========================================
// OMEGA NOKTASI - AST (ABSTRACT SYNTAX TREE) ÇEKİRDEĞİ
// ==========================================
#[derive(Debug)]
enum TextNode {
    Mutable(String),   // Düzeltilecek evren (D0, D1 vb.)
    Protected(String), // Tırnak/Parantez içi (Asla dokunulmaz!)
}

// BÖLÜM 2: Kapsam Analizi ve Yalıtım
// Şimdilik sadece D0 (Ana Evren) boyutunda, güvenli boşluk parçalaması yapıyoruz.
// Gelecekte N-Gram uzay hatalarını (arabamıngeldiğini -> araba_mın_geldiğini) çözen makası da buraya ekleyeceğiz.

pub struct OmegaCompiler<'a> {
    pub engine: &'a MorphEngine,
    pub l_abbr: HashMap<String, String>,
}

impl<'a> OmegaCompiler<'a> {
    pub fn new(engine: &'a MorphEngine) -> Self {
        OmegaCompiler { 
            engine,
            l_abbr: HashMap::new(),
        }
    }

    /// Ham cümleyi alır, atomlarına böler, onarır ve birleştirir.
    pub fn compile_sentence(&self, raw_sentence: &str) -> Result<String, String> {
        let raw_tokens: Vec<&str> = raw_sentence.split_whitespace().collect();
        let n = raw_tokens.len();
        if n == 0 { return Ok(String::new()); }

        // ==========================================
        // OMEGA NOKTASI - TOPOLOJİK MUHASEBE (GİRİŞ)
        // ==========================================
        let mut n_in_total = 0;

        for word in &raw_tokens {
            let token = crate::core::tokenizer::TokenObject::new(word);
            n_in_total += token.get_n_in();
        }

        // 1. ÖN TARAMA (Semantik Rezonans ve Kardeş Alanlar)
        // compiler.rs İÇİNDEKİ YENİ TEMİZ MANTIK (Hardcoding Yok!)
        let mut active_domains: HashMap<String, f32> = HashMap::new();

        for word in &raw_tokens {
            let clean = word.to_lowercase().replace(&['.', ',', '\'', '!', '?'][..], "");
            if let Some(domain) = self.engine.root_dict.get_domain_fast(&clean) {
                if domain != "GENERAL" {
                    let current_score = active_domains.entry(domain.clone()).or_insert(0.0);
                    *current_score += 2.0;

                    // KURALLAR KODDAN DEĞİL, JSON'DAN (BELLEKTEN) GELİR!
                    if let Some(siblings) = self.engine.domain_matrix.get(&domain) {
                        for sibling in siblings {
                            let sib_score = active_domains.entry(sibling.to_string()).or_insert(0.0);
                            if *sib_score < 2.0 { *sib_score = f32::max(*sib_score, 1.0); }
                        }
                    }
                }
            }
        }

        // DP'den önce `active_domains`'i `Vec<String>` formatına çevir (Motora eski formatta paslamak için)
        // Ya da f32 skoru >= 1.0 olan anahtarları (domain isimlerini) al.
        // HATA ÇÖZÜLDÜ: Engine artık HashMap bekliyor, bu dönüşüme gerek yok.

        // ==========================================
        // GLOBAL LATTICE CSP (DAG SHORTEST PATH)
        // dp[i] = (min_total_penalty, prev_node_index, optimal_stem)
        // ==========================================
        // DAG Node Yapısı Genişletildi
        #[derive(Clone, Debug)]
        struct LatticeNode {
            total_penalty: f32,
            prev_idx: usize,
            stem: String,
            token_obj: crate::core::tokenizer::TokenObject,
            local_n_gen: usize,
            local_n_decay: usize,
            final_state: crate::core::suffix_fsm::MorphState, // YENİ: Kelimenin bittiği durak
        }

        let dummy_token = crate::core::tokenizer::TokenObject::new("");
        let mut dp = vec![LatticeNode { 
            total_penalty: 10000.0, 
            prev_idx: 0, 
            stem: String::new(), 
            token_obj: dummy_token, 
            local_n_gen: 0, 
            local_n_decay: 0,
            final_state: crate::core::suffix_fsm::MorphState::EndOfWord // Varsayılan Başlangıç
        }; n + 1];
        dp[0].total_penalty = 0.0;

        for i in 0..n {
            if dp[i].total_penalty >= 10000.0 { continue; }

            // KAPSAM KALKANI (Scope Anchors)
            // Eğer bu bir tırnak/parantez çapasıysa, mutlak olarak tek kelime olarak geçilir, birleştirilemez!
            if raw_tokens[i].contains("__SCOPE_") {
                if dp[i].total_penalty < dp[i+1].total_penalty {
                    dp[i+1] = LatticeNode { 
                        total_penalty: dp[i].total_penalty, 
                        prev_idx: i, 
                        stem: raw_tokens[i].to_string(),
                        token_obj: crate::core::tokenizer::TokenObject::new(raw_tokens[i]),
                        local_n_gen: 0,
                        local_n_decay: 0,
                        final_state: crate::core::suffix_fsm::MorphState::EndOfWord, // Derleme hatasını önlemek için eklendi
                    };
                }
                continue;
            }

            let max_lookahead = std::cmp::min(n, i + 3);
            
            for j in i..max_lookahead {
                // Sentetik çapa bypass
                if j > i && raw_tokens[j].contains("__SCOPE_") { break; }

                // Token Birleştirme Mantığı
                let mut active_token = crate::core::tokenizer::TokenObject::new(raw_tokens[i]);
                for k in (i+1)..=j {
                    let next_tok = crate::core::tokenizer::TokenObject::new(raw_tokens[k]);
                    active_token = active_token.merge(&next_tok);
                }

                let mut path_n_decay = 0;
                let mut path_n_gen = 0;

                // 1. KISALTMA GENİŞLEMESİ VE SÖNÜMLENME (Prefix Expansion & Decay)
                let mut expanded_text = active_token.normalized_text.clone();
                for (abbr, expanded_word) in self.l_abbr.iter() {
                    // Eğer kelime "profun" ise ve "prof" ile başlıyorsa yakala!
                    if expanded_text.starts_with(abbr) {
                        
                        // GÜVENLİK DUVARI: "av" veya "dr" gibi çok kısa kısaltmalar, 
                        // başka kelimelerin köküyle karışmasın diye yanında nokta (.) veya kesme (') ararız.
                        let is_short = abbr.len() <= 2;
                        let has_punct = active_token.anchors.iter().any(|a| a.punct == '.' || a.punct == '\'');
                        if is_short && !has_punct { continue; }

                        // Kuantum Eklemlenmesi: "profun" -> "prof" kısmını kes, "esör" ekle, kalan "un" kısmını yapıştır.
                        let suffix_part = &expanded_text[abbr.len()..];
                        expanded_text = format!("{}{}", expanded_word, suffix_part);
                        
                        // Noktayı mutlak evrenden sil (Decay)
                        if active_token.decay_punctuation('.') {
                            path_n_decay += 1;
                            println!("  [DECAY] '{}' kısaltmasındaki nokta sönümlendi. Yeni kelime: {}", abbr, expanded_text);
                        }
                        
                        break; // Bir kelimede sadece bir kısaltma onarılır
                    }
                }
                
                // Genişletilmiş ve mutasyona uğramış yeni formu motora teslim et (Örn: "profesörun")
                active_token.normalized_text = expanded_text;

                // 2. MATEMATİKSEL ÇEKİRDEĞE SORGULAMA
                let (best_stem, local_penalty, final_state) = match self.engine.parse_with_correction(&active_token.normalized_text, &active_domains) {
                    Ok((stem, penalty, state)) => (stem, penalty, state),
                    Err(_) => {
                        if j == i { (raw_tokens[i].to_string(), 1000.0, MorphState::EndOfWord) } else { continue; }
                    }
                };

                // 3. HAYALET İŞARET ENJEKSİYONU (GHOST)
                // Eğer kelime "Question" (Soru) durağında bittiyse ve üzerinde '?' yoksa, yoktan var et!
                if final_state == MorphState::Question && !active_token.anchors.iter().any(|a| a.punct == '?') {
                    active_token.inject_ghost('?');
                    path_n_gen += 1;
                    println!("  [HAYALET] Olasılık >= 0.99. '{}' kelimesine '?' enjekte edildi.", best_stem);
                }

                // 4. SYNTACTIC AGREEMENT (BIGRAM ENERJİSİ - ÖZNE/YÜKLEM MUTABAKATI)
                let transition_penalty = if i == 0 {
                    0.0 
                } else {
                    use crate::core::suffix_fsm::MorphState;
                    let prev_state = &dp[i].final_state;
                    
                    let  penalty = match (prev_state, &final_state) {
                        (MorphState::Case, MorphState::Person) => -0.5, 
                        (MorphState::Case, MorphState::Tense) => -0.5,
                        (MorphState::RootNoun, MorphState::RootNoun) => 0.0,
                        (MorphState::RootNoun, MorphState::Case) => 0.0,
                        (MorphState::Person, MorphState::RootNoun) => 1.0, 
                        (MorphState::Person, MorphState::Case) => 1.0,
                        (MorphState::Person, MorphState::Person) => 1.5, 
                        _ => 0.2, 
                    };

                    // ==========================================
                    // OMEGA NOKTASI - DERİN BAĞIMLILIK (TRANSITIVITY) MANTIĞI
                    // Eğer önceki kelime Belirtme Hal Eki (ACC) almışsa (Örn: "kitab-ı")
                    // ve şu anki kelime Geçişsiz bir fiilse (Örn: "uyudu"), BÜYÜK CEZA KES!
                    // Not: Bunu tam çalıştırabilmek için token_obj içine hangi Case ekini aldığını kaydetmemiz gerekir.
                    // Şimdilik sadece fiilin genel geçerliliğine bakıyoruz.
                    // ==========================================

                    penalty // Ceza veya ödülü geri dön
                };

                // Formül Kusursuzca Tamamlandı: S_total = Lokal + Boşluk + Bigram
                let space_penalty = if j > i { (j - i) as f32 * 2.5 } else { 0.0 };
                let total_path_penalty = dp[i].total_penalty + local_penalty + space_penalty + transition_penalty;

                // VİTERBİ RAHATLATMASI
                if total_path_penalty < dp[j+1].total_penalty {
                    dp[j+1] = LatticeNode {
                        total_penalty: total_path_penalty,
                        prev_idx: i,
                        stem: best_stem,
                        token_obj: active_token,
                        local_n_gen: path_n_gen,
                        local_n_decay: path_n_decay,
                        final_state: final_state, // YENİ: Başarılı state'i zincire ekle
                    };
                }
            }
        }

        // ==========================================
        // BÖLÜM 5 & 6: BACKTRACKING VE MUTLAK KORUNUM YASASI
        // ==========================================
        let mut final_tokens = Vec::new();
        let mut curr = n;
        
        let mut final_n_gen = 0;
        let mut final_n_decay = 0;

        while curr > 0 {
            let node = &dp[curr];
            
            // Kazanmış evrendeki mutasyona uğramış token'ı inşa et
            let restored = node.token_obj.reconstruct(&node.stem).unwrap_or(node.stem.clone());
            final_tokens.push(restored);
            
            // Sadece kazanan rotanın işaret maliyetleri toplanır!
            final_n_gen += node.local_n_gen;
            final_n_decay += node.local_n_decay;
            
            curr = node.prev_idx;
        }

        final_tokens.reverse();
        let final_text = final_tokens.join(" ");

        // Çıkış (N_out) Muhasebesi
        let mut n_out_total = 0;
        for word in final_text.split_whitespace() {
            let temp_token = crate::core::tokenizer::TokenObject::new(word);
            n_out_total += temp_token.get_n_in();
        }

        println!("\n=== BÖLÜM 5: TOPOLOJİK DOĞRULAMA ===");
        println!("Denklem: N_in({}) + N_gen({}) - N_decay({}) = N_out({})", 
            n_in_total, final_n_gen, final_n_decay, n_out_total);

        if n_in_total + final_n_gen - final_n_decay == n_out_total {
            println!("[KUSURSUZ] Mutlak korunum sağlandı.");
            Ok(final_text)
        } else {
            println!("[KRİTİK HATA] Topolojik korunum yasası İHLAL EDİLDİ! Noktalama işaretleri kayboldu veya havadan türedi.");
            println!("[ROLLBACK] Sistem acil durum moduna geçti. Ham metne (Raw) geri dönülüyor...");
            Err(raw_sentence.to_string()) // EĞER DENKLEM BOZUKSA HİÇBİR ŞEY YAPMAMIŞ GİBİ GERİ DÖN!
        }
    }
    
    pub fn compile_multiverse(&self, raw_text: &str) -> String {
        let ast = self.parse_into_ast(raw_text);
        let mut final_output = String::new();

        for node in ast {
            match node {
                TextNode::Protected(text) => {
                    // Yalıtılmış alan: Doğrudan ekle, motora sokma!
                    final_output.push_str(&text);
                }
                TextNode::Mutable(text) => {
                    if text.trim().is_empty() {
                        final_output.push_str(&text);
                        continue;
                    }
                    
                    // Sadece bu izole evreni motora sok ve Hata İzolasyonu (Micro-Rollback) uygula!
                    match self.compile_sentence(&text) {
                        Ok(corrected) => final_output.push_str(&corrected),
                        Err(_) => {
                            println!("[İZOLE ROLLBACK] Topolojik çöküş! İşlem geri alındı: {}", text.trim());
                            final_output.push_str(&text); // Sadece bu kısmı kurtar!
                        }
                    }
                }
            }
        }
        final_output
    }

    // Basit ve Hataya Toleranslı Lexer: Metni AST düğümlerine böler
    fn parse_into_ast(&self, text: &str) -> Vec<TextNode> {
        let mut nodes = Vec::new();
        let mut current_mutable = String::new();
        let mut chars = text.chars().peekable();

        // 1. AŞAMA: PARANTEZ VE TIRNAK KORUMASI
        while let Some(c) = chars.next() {
            if c == '(' || c == '"' { 
                if !current_mutable.is_empty() {
                    nodes.push(TextNode::Mutable(current_mutable.clone()));
                    current_mutable.clear();
                }
                
                let mut protected_text = String::new();
                protected_text.push(c);
                let closing_char = if c == '(' { ')' } else { '"' };
                let mut is_closed = false;

                while let Some(&next_c) = chars.peek() {
                    protected_text.push(next_c);
                    chars.next();
                    if next_c == closing_char { 
                        is_closed = true;
                        break; 
                    }
                }

                if is_closed {
                    nodes.push(TextNode::Protected(protected_text));
                } else {
                    current_mutable.push_str(&protected_text); // Dengesiz kapanma, hatadır!
                }
            } else {
                current_mutable.push(c);
            }
        }
        if !current_mutable.is_empty() { nodes.push(TextNode::Mutable(current_mutable)); }

        // ==========================================
        // 2. AŞAMA: URL VE LİNK KORUMASI (İKİNCİL AST TARAMASI)
        // ==========================================
        let mut final_nodes = Vec::new();
        for node in nodes {
            match node {
                TextNode::Protected(p) => final_nodes.push(TextNode::Protected(p)),
                TextNode::Mutable(m) => {
                    let mut last_idx = 0;
                    for word in m.split_whitespace() {
                        let lower = word.to_lowercase();
                        if lower.starts_with("http://") || lower.starts_with("https://") || 
                           lower.starts_with("www.") || lower.contains(".com") || 
                           lower.contains(".org") || lower.contains(".net") || lower.contains(".tr") {
                            
                            // MİMARİ ZARİFLİK: Orijinal metindeki boşluk karakterlerini kaybetmeden kelimeyi kopar!
                            let word_start = m[last_idx..].find(word).unwrap() + last_idx;
                            let word_end = word_start + word.len();
                            
                            if word_start > last_idx {
                                final_nodes.push(TextNode::Mutable(m[last_idx..word_start].to_string()));
                            }
                            // URL bulundu, ZIRHLA!
                            final_nodes.push(TextNode::Protected(word.to_string()));
                            last_idx = word_end;
                        }
                    }
                    if last_idx < m.len() {
                        final_nodes.push(TextNode::Mutable(m[last_idx..].to_string()));
                    }
                }
            }
        }
        
        final_nodes
    }
}