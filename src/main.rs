mod core;
use core::root_trie::RootTrie;
use core::morph_engine::MorphEngine;
use core::loader::{load_fsm_from_json, load_roots_from_json};
use core::compiler::OmegaCompiler;

fn main() {
    println!("Omega Noktası - Cümle İşleyici (Compiler Pipeline) Aktif...\n");

    let mut engine = MorphEngine::new(RootTrie::new());
    load_fsm_from_json(&mut engine, "suffixes.json");
    load_roots_from_json(&mut engine, "roots.json"); 

    let compiler = OmegaCompiler::new(&engine);

    // TEST 1: Koca Bir Cümle!
    // İçinde büyük harfler, noktalama işaretleri, fazladan boşluklar ve FSM hataları (yapılmeyacakmı, tanıdıklarımızden) var.
let raw_sentence = "yarın sabah seni de oraya götürücem";
    
    compiler.compile_multiverse(raw_sentence);
}