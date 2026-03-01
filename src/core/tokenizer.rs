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

// 1. Yeni Case Format Enum'ı
#[derive(Debug, Clone, PartialEq)]
pub enum CaseFormat {
    LowerCase,
    TitleCase,
    UpperCase,
    MixedCase(Vec<bool>), // Her harfin büyük/küçük durumunu tutar (Örn: McDonald)
}

#[derive(Debug, Clone)]
pub struct Anchor {
    pub char_index: usize,
    pub orig_index: usize,
    pub punct: char,
    pub status: AnchorStatus,
    pub pos: AnchorPos, // Yön eklendi
}

// 2. TokenObject struct'ındaki string'i bu enum ile değiştir
#[derive(Debug, Clone)]
pub struct TokenObject {
    pub normalized_text: String,
    pub anchors: Vec<Anchor>,
    pub case_format: CaseFormat, // YENİ
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

        // KUSURSUZ VAKA ANALİZİ
        let alpha_chars: Vec<char> = raw_text.chars().filter(|c| c.is_alphabetic()).collect();
        let case_format = if alpha_chars.is_empty() {
            CaseFormat::LowerCase
        } else if alpha_chars.iter().all(|c| c.is_lowercase()) {
            CaseFormat::LowerCase
        } else if alpha_chars.iter().all(|c| c.is_uppercase()) {
            CaseFormat::UpperCase
        } else if alpha_chars[0].is_uppercase() && alpha_chars[1..].iter().all(|c| c.is_lowercase()) {
            CaseFormat::TitleCase
        } else {
            CaseFormat::MixedCase(alpha_chars.iter().map(|c| c.is_uppercase()).collect())
        };

        TokenObject { normalized_text: normalized, anchors, case_format }
    }

    pub fn get_n_in(&self) -> usize {
        self.anchors.len()
    }

    // YENİ: Dışarıdan hayalet (Ghost) işaret enjekte et
    pub fn inject_ghost(&mut self, punct: char) {
        self.anchors.push(Anchor {
            char_index: self.normalized_text.chars().count(), // Kelimenin en sonuna ekle
            orig_index: self.anchors.len(), // Struct kuralı
            punct,
            status: AnchorStatus::Ghost,
            pos: AnchorPos::Trailing, // Hayaletler genelde kelime sonuna eklenir (Örn: kesme işareti)
        });
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
            case_format: self.case_format.clone(), // İlk kelimenin formatı korunur
        }
    }

    pub fn reconstruct(&self, corrected_stem: &str) -> Result<String, String> {
        // Ghost ve Active olanları al, Decayed olanları (Sönümlenmiş) uçur!
        let mut active_anchors: Vec<&Anchor> = self.anchors.iter()
            .filter(|a| a.status != AnchorStatus::Decayed)
            .collect();
        
        // Zafiyet Kapatıldı: Sondan başa doğru ekle ki indeksler kaymasın!
        active_anchors.sort_by(|a, b| {
            b.char_index.cmp(&a.char_index).then(b.orig_index.cmp(&a.orig_index))
        });

        let mut final_chars: Vec<char> = corrected_stem.chars().collect();

        // Formatı geri yükle (Büyük/Küçük harf koruması)
        match &self.case_format {
            CaseFormat::UpperCase => {
                final_chars = final_chars.into_iter()
                    .map(|c| match c { 'ı' => 'I', 'i' => 'İ', _ => c.to_uppercase().next().unwrap_or(c) })
                    .collect();
            },
            CaseFormat::TitleCase => {
                if !final_chars.is_empty() {
                    let first = final_chars[0];
                    final_chars[0] = match first { 'ı' => 'I', 'i' => 'İ', _ => first.to_uppercase().next().unwrap_or(first) };
                }
            },
            CaseFormat::MixedCase(mask) => {
                // Özel harf formatı (Eğer düzeltilmiş kelime orijinalinden çok sapmadıysa maskeyi uygula)
                for i in 0..std::cmp::min(final_chars.len(), mask.len()) {
                    if mask[i] {
                        let c = final_chars[i];
                        final_chars[i] = match c { 'ı' => 'I', 'i' => 'İ', _ => c.to_uppercase().next().unwrap_or(c) };
                    }
                }
            },
            CaseFormat::LowerCase => {} // Zaten küçük harf
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