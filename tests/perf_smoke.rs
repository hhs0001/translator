//! Rough performance guardrails for the subtitle pipeline. The bounds are
//! generous (these run in debug builds) but catch order-of-magnitude
//! regressions in parsing and in the text formatting the editor does per file.

use std::time::Instant;

use translator::core::subtitle::{SubtitleFile, SubtitleFormat};
use translator::core::text_cleaner::{display_text, format_range};

const TAGGED: &str = r"{\an8\pos(640,120)\blur3}Line with tags\Nand a second row";

fn sample_srt(count: usize) -> String {
    (1..=count)
        .map(|i| {
            format!(
                "{i}\n00:00:{:02},000 --> 00:00:{:02},900\n{TAGGED} {i}\n",
                i % 60,
                (i + 1) % 60
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn parsing_and_formatting_two_thousand_lines_is_fast() {
    let content = sample_srt(2000);

    let started = Instant::now();
    let file = SubtitleFile::parse(&content, SubtitleFormat::Srt).expect("parse");
    let parse_ms = started.elapsed().as_millis();
    assert_eq!(file.entries.len(), 2000);

    // This is what the editor does once per file when building its row cache.
    let started = Instant::now();
    let formatted: usize = file
        .entries
        .iter()
        .map(|entry| {
            let text = display_text(&entry.text);
            let time = format_range(&entry.start_time, &entry.end_time);
            text.len() + time.len()
        })
        .sum();
    let format_ms = started.elapsed().as_millis();
    assert!(formatted > 0);

    println!("parse: {parse_ms} ms, format: {format_ms} ms");
    assert!(parse_ms < 1500, "parsing 2000 lines took {parse_ms} ms");
    assert!(
        format_ms < 1500,
        "formatting 2000 lines took {format_ms} ms"
    );
}

#[test]
fn tags_are_stripped_for_display() {
    let raw = r"{\an8\pos(640,120)\blur3}TOKYO - 07:42\NSecond line";
    assert_eq!(display_text(raw), "TOKYO - 07:42\nSecond line");

    let karaoke = r"{\k30}La{\k22}la{\k40}la, canta comigo";
    assert_eq!(display_text(karaoke), "Lalala, canta comigo");

    // The leading hour is dropped, so timecodes read as mm:ss.cc.
    assert_eq!(
        format_range("0:00:05.00", "0:00:09.00"),
        "00:05.00  –  00:09.00"
    );
    assert_eq!(
        format_range("00:28:24,000", "00:28:27,000"),
        "28:24.000  –  28:27.000"
    );
}
