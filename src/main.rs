mod core;
use core::root_trie::RootTrie;
use core::morph_engine::MorphEngine;
use core::loader::{load_fsm_from_json, load_roots_from_json};
use core::compiler::OmegaCompiler;

fn main() {
    println!("==================================================");
    println!("OMEGA NOKTASI - ÇEKİRDEK BAŞLATILIYOR...");
    println!("==================================================\n");

    // 1. Motoru ve Belleği İnşa Et
    let mut engine = MorphEngine::new(RootTrie::new());
    
    // 2. Veritabanlarını (Beyni) Yükle
    load_fsm_from_json(&mut engine, "suffixes.json");
    load_roots_from_json(&mut engine, "roots.json"); 

    // Not: Eğer kısaltma sözlüğünü (abbreviations.json) loader'a bağladıysan 
    // ve OmegaCompiler::new() onu parametre olarak bekliyorsa, buraya ekleyebilirsin.
    // Şimdilik standart kurucu ile devam ediyoruz.
    let compiler = OmegaCompiler::new(&engine);

    // 3. Ham Girdi (Kullanıcı Verisi)
    let raw_sentence = "bugun evraga aykiri islem yapinca saatinda ormana kacucam";
    
    println!("[GİRDİ]: {}", raw_sentence);
    println!("--------------------------------------------------");

    // 4. Orkestratörü Ateşle!
    // Hiyerarşi: Multiverse (Parçalar) -> Sentence (DAG Matrisi) -> FSM -> Phonology
    let final_output = compiler.compile_multiverse(raw_sentence);

    // 5. Mutlak Çıktı
    println!("--------------------------------------------------");
    println!("[MUTLAK ÇIKTI]: {}", final_output);
    println!("==================================================\n");
}