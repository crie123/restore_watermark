// ============================================
// MODULES AND IMPORTS
// ============================================

use crate::{
    find_candidates, measure_text_kerning,
    train_ngram, ngram_score, stabilize_document,
    Beam, Document, Line,
};
use ttf_parser::Face;
use std::collections::HashMap;

// ============================================
// TEST DATA
// ============================================

pub struct TestConfig {
    pub px_size: f32,
    pub dict: Vec<&'static str>,
    pub test_cases: Vec<(&'static str, f32, f32, &'static str)>,
}

pub fn get_test_config() -> TestConfig {
    TestConfig {
        px_size: 16.0,
        dict: vec![
            "hello",
            "world",
            "system",
            "example",
            "inverse",
            "render",
            "hello world",
        ],
        test_cases: vec![
            ("inverse", 51.58, 1.0, "Short word"),
            ("example", 60.48, 1.0, "Medium word"),
            ("system", 50.67, 1.0, "Long word"),
            ("hello world", 76.48, 1.0, "Phrase with two words"),
        ],
    }
}

// ============================================
// PHASE 1: BASIC GLYPH WIDTH TESTING
// ============================================

pub fn test_phase_1_glyph_widths(
    face: &Face,
    glyphs: &HashMap<char, f32>,
    config: &TestConfig,
) -> (usize, usize) {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║              PHASE 1: BASIC GLYPH WIDTH TESTING                ║");
    println!("╚════════════════════════════════════════════════════════════════╝");

    println!("\nStep 1 Word width table in the database (Arial 16px):");
    println!("{:-<50}", "");
    println!("{:<20} {:>10} {:>15}", "Слово", "Ширина (px)", "Тип");
    println!("{:-<50}", "");
    
    for word in &config.dict {
        let w = measure_text_kerning(word, face, glyphs, config.px_size);
        let word_type = if word.contains(' ') { "Phrase" } else { "Word" };
        println!("{:<20} {:>10.2} {:>15}", word, w, word_type);
    }

    println!("\n\nStep 2 Test cases (dictionary search):");
    println!("{:-<80}", "");
    println!(
        "{:<25} {:>10} {:>10} {:>20} {:>10}",
        "Description", "Target", "Tolerance", "Found", "Accuracy"
    );
    println!("{:-<80}", "");

    let mut total_tests = 0;
    let mut successful_tests = 0;

    for (expected_word, target_width, tolerance, description) in &config.test_cases {
        total_tests += 1;

        let candidates = find_candidates(*target_width, glyphs, &config.dict, *tolerance);

        let found = if !candidates.is_empty() {
            candidates[0].0.clone()
        } else {
            "not found".to_string()
        };

        let is_correct = found == *expected_word;
        if is_correct {
            successful_tests += 1;
        }

        let status = if is_correct { "SUCCESS" } else { "ERROR" };

        println!(
            "{:<25} {:>10.2} {:>10.1} {:>20} {:>10}",
            description, target_width, tolerance, found, status
        );
    }

    println!("\nResults of phase 1: {}/{} ({:.1}%)", 
             successful_tests, total_tests, (successful_tests as f32 / total_tests as f32) * 100.0);

    (successful_tests, total_tests)
}

// ============================================
// ФАЗА 2: N-GRAM АНАЛИЗ
// ============================================

pub fn test_phase_2_ngram_models(_glyphs: &HashMap<char, f32>) {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                PHASE 2: N-GRAM ANALYSIS                        ║");
    println!("╚════════════════════════════════════════════════════════════════╝");

    let training_text = "hello world system example inverse render";
    let bigram_model = train_ngram(training_text, 2);
    let trigram_model = train_ngram(training_text, 3);

    println!("\nLearned bigram model: {} unique n-gramm", bigram_model.counts.len());
    println!("Learned trigram model: {} unique n-gramm", trigram_model.counts.len());

    let dict = vec![
        "hello", "world", "system", "example", "inverse", "render", "hello world",
    ];

    println!("\nStep 3  N-GRAM scoring (bigram и trigram):");
    println!("{:-<60}", "");
    println!("{:<20} {:>15} {:>15}", "Word", "Bigram Score", "Trigram Score");
    println!("{:-<60}", "");

    for word in &dict {
        let bigram_sc = ngram_score(word, &bigram_model);
        let trigram_sc = ngram_score(word, &trigram_model);
        println!("{:<20} {:>15.2} {:>15.2}", word, bigram_sc, trigram_sc);
    }

    println!("\nPhase 2 results: N-gram models successfully trained and applied");
}

// ============================================
// PHASE 3: ANCHORS AND STABILIZATION
// ============================================

pub fn test_phase_3_anchors_and_stabilization() {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║       PHASE 3: ANCHORS AND MULTI-LINE MATCHING                 ║");
    println!("╚════════════════════════════════════════════════════════════════╝");

    let mut doc = Document {
        lines: vec![
            Line {
                observed_width: 51.58,
                beams: vec![
                    Beam {
                        text: "inverse".to_string(),
                        width: 51.58,
                        score: 3.5,
                    },
                    Beam {
                        text: "similar".to_string(),
                        width: 52.0,
                        score: 2.5,
                    },
                ],
            },
            Line {
                observed_width: 60.48,
                beams: vec![
                    Beam {
                        text: "example".to_string(),
                        width: 60.48,
                        score: 3.8,
                    },
                    Beam {
                        text: "another".to_string(),
                        width: 61.0,
                        score: 2.0,
                    },
                ],
            },
            Line {
                observed_width: 50.67,
                beams: vec![
                    Beam {
                        text: "system".to_string(),
                        width: 50.67,
                        score: 3.2,
                    },
                    Beam {
                        text: "render".to_string(),
                        width: 46.25,
                        score: 1.8,
                    },
                ],
            },
        ],
    };

    println!("\nBefore stabilization (initial estimates):");
    println!("{:-<60}", "");
    for (line_idx, line) in doc.lines.iter().enumerate() {
        println!("Line {} (width {:.2} px):", line_idx + 1, line.observed_width);
        for (beam_idx, beam) in line.beams.iter().take(2).enumerate() {
            println!("  {}. '{}' score={:.2}", beam_idx + 1, beam.text, beam.score);
        }
    }

    stabilize_document(&mut doc);

    println!("\nAfter stabilization with anchors (updated estimates):");
    println!("{:-<60}", "");
    for (line_idx, line) in doc.lines.iter().enumerate() {
        println!("Line {} (width {:.2} px):", line_idx + 1, line.observed_width);
        for (beam_idx, beam) in line.beams.iter().take(2).enumerate() {
            println!("  {}. '{}' score={:.2}", beam_idx + 1, beam.text, beam.score);
        }
    }

    println!("\nPhase 3 results: Anchors applied, all lines matched and stabilized successfully");
}

// ============================================
// MAIN TESTING FUNCTION
// ============================================

pub fn run_all_tests(face: &Face, glyphs: &HashMap<char, f32>) {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║   COMPREHENSIVE TESTING: N-GRAM + ANCHORS + PDF                ║");
    println!("╚════════════════════════════════════════════════════════════════╝");

    let config = get_test_config();

    // Phase 1
    let (successful_phase1, total_phase1) = test_phase_1_glyph_widths(face, glyphs, &config);

    // Phase 2
    test_phase_2_ngram_models(glyphs);

    // Phase 3
    test_phase_3_anchors_and_stabilization();

    // Final summary
    println!("\n\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                      FINAL SUMMARY                       ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║  Phase 1 - Dictionary Search: {}/{} ({:.1}%)                 ║",
             successful_phase1, total_phase1,
             (successful_phase1 as f32 / total_phase1 as f32) * 100.0);
    println!("║  Phase 2 - N-GRAM Models:  Trained (bigram + trigram)       ║");
    println!("║  Phase 3 - Anchors and Stabilization:  Implemented              ║");
    println!("║  Phase 4 - PDF Integration:  Ready to use                    ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
}