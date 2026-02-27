#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub enum AnchorStatus {
    Active,
    Decayed,
    Ghost,
}

// YENİ: Noktalamanın evrendeki yönü
#[derive(Debug, Clone, PartialEq)]
pub enum AnchorPos {
    Leading,  // Kelimenin başında (Örn: Parantez, Tırnak)
    Trailing, // Kelimenin içinde veya sonunda (Örn: Kesme, Nokta, Virgül)
}

#[derive(Debug, Clone)]
pub struct Anchor {
    pub char_index: usize,
    pub orig_index: usize,
    pub punct: char,
    pub status: AnchorStatus,
    pub pos: AnchorPos, // Yön eklendi
}

#[derive(Debug, Clone)]
pub struct TokenObject {
    pub normalized_text: String,
    pub anchors: Vec<Anchor>,
    pub case_flag: String,
}

impl TokenObject {
    pub fn new(raw_text: &str) -> Self {
        let mut normalized = String::new();
        let mut anchors = Vec::new();
        let mut current_char_index = 0;
        let mut chars_processed = 0; // Harf gördük mü? (Leading/Trailing ayrımı için)

        for ch in raw_text.chars() {
            if ch.is_alphabetic() {
                // 1. KRİTİK YAMA: Rust'ın Türkçe Unicode Körlüğü Giderildi
                let lower_ch = match ch {
                    'I' => 'ı',
                    'İ' => 'i',
                    _ => ch.to_lowercase().next().unwrap_or(ch),
                };
                normalized.push(lower_ch);
                current_char_index += 1;
                chars_processed += 1;
            } else if ch.is_numeric() {
                normalized.push(ch);
                current_char_index += 1;
                chars_processed += 1;
            } else {
                // 2. KRİTİK YAMA: Öncül/Ardıl Çapa Yönü Tespiti
                let pos = if chars_processed == 0 { 
                    AnchorPos::Leading 
                } else { 
                    AnchorPos::Trailing 
                };

                anchors.push(Anchor {
                    char_index: current_char_index,
                    orig_index: anchors.len(),
                    punct: ch,
                    status: AnchorStatus::Active,
                    pos,
                });
            }
        }

        let case_flag = if raw_text.chars().next().unwrap_or(' ').is_uppercase() {
            "TitleCase".to_string()
        } else {
            "LowerCase".to_string()
        };

        TokenObject { normalized_text: normalized, anchors, case_flag }
    }

    pub fn get_n_in(&self) -> usize {
        self.anchors.len()
    }

    // 3. KRİTİK YAMA: Tokenleri String olarak değil, Vektör olarak birleştir (İndeksleri koru)
    pub fn merge(&self, other: &TokenObject) -> TokenObject {
        let merged_normalized = format!("{}{}", self.normalized_text, other.normalized_text);
        let mut merged_anchors = self.anchors.clone();
        
        let offset = self.normalized_text.chars().count();
        let orig_offset = self.anchors.len();
        
        for mut anchor in other.anchors.clone() {
            if anchor.pos == AnchorPos::Leading {
                // İkinci kelimenin başındaki işaret, birleşince kelimenin İÇİNE düşer
                anchor.pos = AnchorPos::Trailing; 
            }
            anchor.char_index += offset;
            anchor.orig_index += orig_offset;
            merged_anchors.push(anchor);
        }
        
        TokenObject {
            normalized_text: merged_normalized,
            anchors: merged_anchors,
            case_flag: self.case_flag.clone(), // İlk kelimenin formatı korunur
        }
    }

    pub fn reconstruct(&self, corrected_stem: &str) -> Result<String, String> {
        let mut active_anchors: Vec<&Anchor> = self.anchors.iter()
            .filter(|a| a.status == AnchorStatus::Active)
            .collect();
        
        // Zafiyet Kapatıldı: Sondan başa doğru ekle ki indeksler kaymasın!
        active_anchors.sort_by(|a, b| {
            b.char_index.cmp(&a.char_index).then(b.orig_index.cmp(&a.orig_index))
        });

        let mut final_chars: Vec<char> = corrected_stem.chars().collect();

        // Formatı geri yükle (Büyük/Küçük harf koruması)
        if self.case_flag == "TitleCase" && !final_chars.is_empty() {
            let first_char = final_chars[0];
            let upper_char = match first_char {
                'ı' => 'I',
                'i' => 'İ',
                _ => first_char.to_uppercase().next().unwrap_or(first_char),
            };
            final_chars[0] = upper_char;
        }

        for anchor in active_anchors {
            if anchor.pos == AnchorPos::Leading {
                // Leading ise kesinlikle en başa koy!
                final_chars.insert(0, anchor.punct);
            } else {
                // Trailing ise yerine (veya sınırların dışına taşmamasına dikkat ederek) koy!
                let insert_pos = std::cmp::min(anchor.char_index, final_chars.len());
                final_chars.insert(insert_pos, anchor.punct);
            }
        }

        Ok(final_chars.into_iter().collect())
    }
    // BÖLÜM 4: SÖNÜMLENME YETENEĞİ (Decay)
    pub fn decay_punctuation(&mut self, target_punct: char) -> bool {
        // Sondan başa doğru ara ki, "Prof.'un" içindeki son noktayı (kesmeyi değil) bulsun
        for anchor in self.anchors.iter_mut().rev() {
            if anchor.punct == target_punct && anchor.status == AnchorStatus::Active {
                anchor.status = AnchorStatus::Decayed;
                return true;
            }
        }
        false
    }
}