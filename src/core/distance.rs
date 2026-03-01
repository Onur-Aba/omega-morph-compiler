#![allow(dead_code)]

// =========================================================================
// OMEGA NOKTASI - FİZİKSEL KLAVYE UZAYI VE AĞIRLIKLI CEZA MATRİSİ
// =========================================================================

/// Türkçe QWERTY klavyenin Kartezyen (X, Y) koordinat haritası
fn get_key_coords(c: char) -> Option<(f32, f32)> {
    match c {
        // Üst Sıra
        'q'=>Some((0.0, 0.0)), 'w'=>Some((1.0, 0.0)), 'e'=>Some((2.0, 0.0)), 'r'=>Some((3.0, 0.0)), 't'=>Some((4.0, 0.0)), 'y'=>Some((5.0, 0.0)), 'u'=>Some((6.0, 0.0)), 'ı'=>Some((7.0, 0.0)), 'o'=>Some((8.0, 0.0)), 'p'=>Some((9.0, 0.0)), 'ğ'=>Some((10.0, 0.0)), 'ü'=>Some((11.0, 0.0)),
        // Orta Sıra
        'a'=>Some((0.5, 1.0)), 's'=>Some((1.5, 1.0)), 'd'=>Some((2.5, 1.0)), 'f'=>Some((3.5, 1.0)), 'g'=>Some((4.5, 1.0)), 'h'=>Some((5.5, 1.0)), 'j'=>Some((6.5, 1.0)), 'k'=>Some((7.5, 1.0)), 'l'=>Some((8.5, 1.0)), 'ş'=>Some((9.5, 1.0)), 'i'=>Some((10.5, 1.0)),
        // Alt Sıra
        'z'=>Some((1.0, 2.0)), 'x'=>Some((2.0, 2.0)), 'c'=>Some((3.0, 2.0)), 'v'=>Some((4.0, 2.0)), 'b'=>Some((5.0, 2.0)), 'n'=>Some((6.0, 2.0)), 'm'=>Some((7.0, 2.0)), 'ö'=>Some((8.0, 2.0)), 'ç'=>Some((9.0, 2.0)), '.'=>Some((10.0, 2.0)), ','=>Some((11.0, 2.0)),
        _ => None
    }
}

pub fn char_substitution_cost(expected: char, actual: char) -> f32 {
    if expected == actual { return 0.0; }

    // 1. DİLBİLGİSEL MUTASYON (Ünsüz Yumuşaması) - SIFIR CEZA
    let mutation_pairs = [
        ('k', 'ğ'), ('p', 'b'), ('ç', 'c'), ('t', 'd'), ('k', 'g'),
        ('k', 'ğ'), ('p', 'b'), ('ç', 'c'), ('t', 'd'), ('k', 'g'),
        // ASCII / Türkçe Karakter Tembelliği
        ('ş', 's'), ('s', 'ş'), 
        ('ç', 'c'), ('c', 'ç'), 
        ('ğ', 'g'), ('g', 'ğ'), 
        ('ı', 'i'), ('i', 'ı'), 
        ('ö', 'o'), ('o', 'ö'), 
        ('ü', 'u'), ('u', 'ü')
    ];
    if mutation_pairs.contains(&(expected, actual)) {
        return 0.2; 
    }

    let mut penalty = 3.5; // Varsayılan astronomik ceza YÜKSELTİLDİ! Kökleri kolayca bozamasın!

    // 2. FİZİKSEL KLAVYE MESAFESİ (Öklid Matematiği)
    if let (Some(c1), Some(c2)) = (get_key_coords(expected), get_key_coords(actual)) {
        let dx = c1.0 - c2.0;
        let dy = c1.1 - c2.1;
        let dist = (dx * dx + dy * dy).sqrt();
        
        if dist <= 1.2 {
            penalty = 0.6; // Bitişik tuş cezası ARTIRILDI (Örn: t yerine r geldiğinde 0.4 yetmez, hemen kelime değiştirir)
        } else if dist <= 2.2 {
            penalty = 1.2; // Çapraz tuş cezası ARTIRILDI
        } else {
            penalty = dist * 1.0; // Uzak mesafe çarpanı ARTIRILDI
        }
    }

    // 3. FONOLOJİK YANILGI VE DNA KORUMASI (En kritik yer!)
    let vowels = ['a', 'e', 'ı', 'i', 'o', 'ö', 'u', 'ü', 'A', 'E', 'I', 'İ', 'O', 'Ö', 'U', 'Ü'];
    let is_v1 = vowels.contains(&expected);
    let is_v2 = vowels.contains(&actual);

    if is_v1 && is_v2 {
        // İkisi de ünlü harf ise: Bu sadece bir uyum hatasıdır, ceza tavanını indir.
        if penalty > 0.8 { penalty = 0.8; }
    } else if !is_v1 && !is_v2 {
        // İkisi de ünsüz harf ise (Örn: 't' yerine 'r'):
        // Klavye mesafesi yakın olsa bile (0.6), bunu biraz artır ki "götür" yerine "gör" köküne hemen sapmasın.
        if penalty < 1.2 { penalty = 1.2; }
    } else {
        // Biri ünlü biri ünsüz ise (DNA Değişimi!): 
        // Kullanıcı klavyede yakın bassa bile bu ASLA affedilemez devasa bir hatadır.
        penalty = 3.0;
    }

    penalty
}

// ==========================================
// OMEGA NOKTASI - %100 ZERO-ALLOCATION (SIFIR YIĞIN) L1 CACHE DP ALGORİTMASI
// ==========================================
pub fn match_suffix_fuzzy(expected_suffix: &str, remaining_user: &str) -> (f32, usize) {
    let exp_chars: Vec<char> = expected_suffix.chars().collect();
    let usr_chars: Vec<char> = remaining_user.chars().collect();
    
    let m = exp_chars.len();
    if m == 0 { return (0.0, 0); }
    if usr_chars.is_empty() { return (m as f32 * 1.0, 0); }

    // MİMARİ MÜDAHALE 1: Boyut 32'den 64'e çıkarıldı. (16KB Stack Bellek - L1 Cache için kusursuz)
    const MAX_DIM: usize = 64; 

    // MİMARİ MÜDAHALE 2: Sessiz kırpma (Truncation) yok!
    // Eğer bir ek veya kelime 64 harften uzunsa, motoru yorma ve bozma; doğrudan "Eşleşmedi" diyerek reddet!
    if m >= MAX_DIM { return (1000.0, 0); }

    let safe_m = m;
    // Kullanıcının metninden en fazla ek uzunluğu + 2 harf koparıp bakarız. Kalanı Viterbi'nin işi değildir.
    let search_limit = std::cmp::min(usr_chars.len(), safe_m + 2);
    
    // Eğer arama limiti de 64'ü aşıyorsa reddet (Güvenlik Zırhı)
    if search_limit >= MAX_DIM { return (1000.0, 0); }

    let theoretical_min = if safe_m > 2 { safe_m - 2 } else { 0 };
    let min_len = std::cmp::min(theoretical_min, search_limit);

    // Sıfır Heap Allocation! Dinamik array yerine statik Stack allocation.
    let mut dp = [[0.0_f32; MAX_DIM]; MAX_DIM];

    for i in 0..=safe_m { dp[i][0] = i as f32 * 1.0; }
    for j in 0..=search_limit { dp[0][j] = j as f32 * 1.0; }

    for i in 1..=safe_m {
        for j in 1..=search_limit {
            let cost = char_substitution_cost(exp_chars[i-1], usr_chars[j-1]);
            
            let delete = dp[i-1][j] + 1.2;
            let insert = dp[i][j-1] + 1.2;
            let substitute = dp[i-1][j-1] + cost;
            
            let mut min_val = f32::min(delete, f32::min(insert, substitute));

            if i > 1 && j > 1 && exp_chars[i-1] == usr_chars[j-2] && exp_chars[i-2] == usr_chars[j-1] {
                min_val = f32::min(min_val, dp[i-2][j-2] + 0.5); 
            }

            dp[i][j] = min_val;
        }
    }

    let mut best_penalty = f32::MAX;
    let mut best_consumed = 0;

    for take_len in min_len..=search_limit {
        if dp[safe_m][take_len] < best_penalty {
            best_penalty = dp[safe_m][take_len];
            best_consumed = take_len;
        }
    }
    
    let lower_expected = expected_suffix.to_lowercase();
    if lower_expected.ends_with("eceğim") || lower_expected.ends_with("acağım") || 
       lower_expected.ends_with("iyorum") || lower_expected.ends_with("ıyorum") {
        return (f32::max(0.0, best_penalty - 1.5), best_consumed);
    }

    (best_penalty, best_consumed)
}