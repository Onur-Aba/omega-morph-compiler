#![allow(dead_code)]

// =========================================================================
// OMEGA NOKTASI - FİZİKSEL KLAVYE UZAYI VE AĞIRLIKLI CEZA MATRİSİ
// =========================================================================

/// Türkçe QWERTY klavyenin Kartezyen (X, Y) koordinat haritası
fn get_key_coords(c: char) -> Option<(f32, f32)> {
    match c {
        // Üst Sıra
        'q'=>Some((0.0, 0.0)), 'w'=>Some((1.0, 0.0)), 'e'=>Some((2.0, 0.0)), 'r'=>Some((3.0, 0.0)), 't'=>Some((4.0, 0.0)), 'y'=>Some((5.0, 0.0)), 'u'=>Some((6.0, 0.0)), 'ı'=>Some((7.0, 0.0)), 'o'=>Some((8.0, 0.0)), 'p'=>Some((9.0, 0.0)), 'ğ'=>Some((10.0, 0.0)), 'ü'=>Some((11.0, 0.0)),
        // Orta Sıra (Yarım birim sağa kayık)
        'a'=>Some((0.5, 1.0)), 's'=>Some((1.5, 1.0)), 'd'=>Some((2.5, 1.0)), 'f'=>Some((3.5, 1.0)), 'g'=>Some((4.5, 1.0)), 'h'=>Some((5.5, 1.0)), 'j'=>Some((6.5, 1.0)), 'k'=>Some((7.5, 1.0)), 'l'=>Some((8.5, 1.0)), 'ş'=>Some((9.5, 1.0)), 'i'=>Some((10.5, 1.0)),
        // Alt Sıra (Bir birim sağa kayık)
        'z'=>Some((1.0, 2.0)), 'x'=>Some((2.0, 2.0)), 'c'=>Some((3.0, 2.0)), 'v'=>Some((4.0, 2.0)), 'b'=>Some((5.0, 2.0)), 'n'=>Some((6.0, 2.0)), 'm'=>Some((7.0, 2.0)), 'ö'=>Some((8.0, 2.0)), 'ç'=>Some((9.0, 2.0)), '.'=>Some((10.0, 2.0)), ','=>Some((11.0, 2.0)),
        _ => None
    }
}

pub fn char_substitution_cost(expected: char, actual: char) -> f32 {
    if expected == actual { return 0.0; }

    // 1. DİLBİLGİSEL MUTASYON (Ünsüz Yumuşaması) - SIFIR CEZA
    let mutation_pairs = [
        ('k', 'ğ'), ('p', 'b'), ('ç', 'c'), ('t', 'd'), ('k', 'g')
    ];
    if mutation_pairs.contains(&(expected, actual)) {
        return 0.0; 
    }

    let mut penalty = 2.5; // Varsayılan astronomik ceza (Uzak harfler FSM'yi patlatır)

    // 2. FİZİKSEL KLAVYE MESAFESİ (Öklid Matematiği)
    if let (Some(c1), Some(c2)) = (get_key_coords(expected), get_key_coords(actual)) {
        let dx = c1.0 - c2.0;
        let dy = c1.1 - c2.1;
        let dist = (dx * dx + dy * dy).sqrt();
        
        if dist <= 1.2 {
            penalty = 0.4; // Bitişik tuşa basmış (Örn: m yerine n veya j)
        } else if dist <= 2.2 {
            penalty = 0.9; // Çapraz veya bir tuş atlamış
        } else {
            penalty = dist * 0.8; // Tuşlar uzaklaştıkça ceza KATLANIR! (m -> y mesafesi = 2.82. Ceza: 2.26)
        }
    }

    // 3. FONOLOJİK YANILGI (Kullanıcı uzağa bassa bile, sadece ses uyumu kuralını bilmiyor olabilir)
    let vowels = ['a', 'e', 'ı', 'i', 'o', 'ö', 'u', 'ü'];
    if vowels.contains(&expected) && vowels.contains(&actual) {
        if penalty > 0.6 {
            penalty = 0.6; // Sesli harflerin kendi arasındaki yanlışlarına tavan limit koy!
        }
    }

    penalty
}

pub fn match_suffix_fuzzy(expected_suffix: &str, remaining_user: &str) -> (f32, usize) {
    let exp_chars: Vec<char> = expected_suffix.chars().collect();
    let usr_chars: Vec<char> = remaining_user.chars().collect();
    
    let m = exp_chars.len();
    if m == 0 { return (0.0, 0); }
    if usr_chars.is_empty() { return (m as f32 * 1.0, 0); } // Eksik harf başına 1 tam ceza

    let mut best_penalty = 1000.0;
    let mut best_consumed = 0;

    // ==========================================
    // KUSURSUZ SINIR KORUMASI (Out of Bounds Fix)
    // ==========================================
    let theoretical_min = if m > 2 { m - 2 } else { 0 };
    // Asla kullanıcının girdiği harf sayısından (usr_chars.len()) fazlasını okumaya çalışma!
    let min_len = std::cmp::min(theoretical_min, usr_chars.len());
    let max_len = std::cmp::min(usr_chars.len(), m + 2);

    for take_len in min_len..=max_len {
        let user_slice = &usr_chars[0..take_len]; // ARTIK ASLA PANİĞE GİRMEYECEK!
        
        // Dinamik Programlama Matrisi (Damerau-Levenshtein)
        let mut dp = vec![vec![0.0; take_len + 1]; m + 1];
        
        for i in 0..=m { dp[i][0] = i as f32 * 1.0; } 
        for j in 0..=take_len { dp[0][j] = j as f32 * 1.0; }
        
        for i in 1..=m {
            for j in 1..=take_len {
                let cost = char_substitution_cost(exp_chars[i-1], user_slice[j-1]);
                
                let delete = dp[i-1][j] + 1.0;
                let insert = dp[i][j-1] + 1.0;
                let substitute = dp[i-1][j-1] + cost;
                
                let mut min_val = f32::min(delete, f32::min(insert, substitute));
                
                // KUSURSUZ TRANSPOZİSYON: Harflerin yeri değişmişse affet! (lü <-> ül)
                if i > 1 && j > 1 && exp_chars[i-1] == user_slice[j-2] && exp_chars[i-2] == user_slice[j-1] {
                    min_val = f32::min(min_val, dp[i-2][j-2] + 0.5); 
                }
                
                dp[i][j] = min_val;
            }
        }
        
        if dp[m][take_len] < best_penalty {
            best_penalty = dp[m][take_len];
            best_consumed = take_len; // Motorun kullanıcının metninden kaç harf tüketeceğini belirler
        }
    }
    
    (best_penalty, best_consumed)
}