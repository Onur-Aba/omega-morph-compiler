#![allow(dead_code)] // Cephaneliği şimdilik susturuyoruz
use serde::Deserialize;

// =========================================================================
// OMEGA NOKTASI - 14-BİT EK BAYRAKLARI (SUFFIX MUTATION FLAGS)
// Köklerin DNA'sıyla etkileşime girecek olan eklere ait genetik kodlar.
// =========================================================================

// 1. Ünlü Uyumu (Harmony Type - 2 Bit)
pub const HARMONY_TWO_WAY: u16   = 1 << 0; // A/E uyumu (lar/ler)
pub const HARMONY_FOUR_WAY: u16  = 1 << 1; // I/İ/U/Ü uyumu (ım/im/um/üm)
pub const HARMONY_O_VARIANT: u16 = 1 << 2; // Sadece O istisnası (-yor)

// 2. Kaynaştırma İhtiyacı (Buffer Letter Requirement - 3 Bit)
pub const BUFFER_Y: u16          = 1 << 3; // İki ünlü arası Y (kapı-y-a)
pub const BUFFER_S: u16          = 1 << 4; // 3. Tekil iyelik S (araba-s-ı)
pub const BUFFER_N: u16          = 1 << 5; // İlgi eki N (Ali-n-in)

// 3. Ünsüz Benzeşmesi (Consonant Mutation - 2 Bit)
pub const MUT_D_T: u16           = 1 << 6; // D ve T arası geçiş (-da/-te)
pub const MUT_C_C: u16           = 1 << 7; // C ve Ç arası geçiş (-cı/-çi)
pub const MUT_G_K: u16           = 1 << 8; // G ve K arası geçiş (-gan/-ken)

// 4. Diğer İstisnai Yıkım ve Üretim Durumları (3 Bit)
pub const CAUSES_SOFTENING: u16  = 1 << 9;  // Geldiği kökü yumuşatır (Örn: -a, -ı)
pub const ADDS_PRON_N: u16       = 1 << 10; // Kendinden sonra hal eki gelirse N türetir
pub const DROPS_INITIAL_I: u16   = 1 << 11; // Bitiştiğinde baştaki i düşer (idi, ile)

// =========================================================================
// OMEGA NOKTASI - FSM DURUMLARI (STATE STATIONS)
// =========================================================================
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)] // Deserialize eklendi!
pub enum MorphState {
    RootNoun,     // İsim Kökü (ve İsimden İsim / Fiilden İsim yapan eklerin çıkışı)
    RootVerb,     // Fiil Kökü (ve İsimden Fiil / Fiilden Fiil yapan eklerin çıkışı)
    Voice,        // Çatı Ekleri (Edilgen, Ettirgen, İşteş)
    Negative,     // Olumsuzluk Eki (-ma/-me)
    Plural,       // Çoğul Eki (-lar/-ler)
    Possessive,   // İyelik Ekleri
    Case,         // Hal Ekleri
    Tense,        // Kip ve Zaman Ekleri
    Person,       // Şahıs Ekleri
    Copula,       // Ek-Fiil (İsmi yüklem yapanlar)
    Question,     // Soru Eki (-mı/-mi)
    EndOfWord,    // Terminal
}

// =========================================================================
// OMEGA NOKTASI - EK İSTASYONU (FSM NODE)
// =========================================================================
#[derive(Debug, Clone)]
pub struct SuffixNode {
    pub id: String,                 // Sistemsel Kimlik: "PLURAL_LER"
    pub base_form: String,          // Fonolojik şablon: "lAr" (Büyük A = 2'li uyum)
    pub flags: u16,                 // 14-bitlik ek DNA'sı
    pub output_state: MorphState,   // Bu ek alındığında geçilen YENİ durum
    pub allowed_next: Vec<MorphState>, // Bu ekten sonra gidilebilecek raylar (Kurallar)
    pub canonical_form: Option<String>, // YENİ: Makro Eklerin resmi genişlemesi!
}

impl SuffixNode {
    /// Deterministik bir Ek İstasyonu inşa eder.
    pub fn new(id: String, base_form: String, flags: u16, output_state: MorphState, allowed_next: Vec<MorphState>) -> Self {
        SuffixNode {
            id,
            base_form,
            flags,
            output_state,
            allowed_next,
            canonical_form: None, // YENİ: Makro Eklerin resmi genişlemesi!
        }
    }
}