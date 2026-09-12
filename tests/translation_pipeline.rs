//! End-to-end checks for the batching/parallel scheduler against a fake
//! OpenAI-compatible server. These cover the behaviour that is easy to break:
//! how many requests are in flight, retries, batch telemetry and cancellation.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use translator::core::translator::{
    BatchStatus, BatchUpdate, LlmClient, LlmConfig, TranslationSettings,
    TRANSLATION_CANCELLED_ERROR,
};

/// How a fake server should answer each incoming request.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Behaviour {
    /// Always echo the lines back, prefixed.
    Ok,
    /// Fail the first `n` requests with a 500, then succeed.
    FailFirst(usize),
}

struct Server {
    port: u16,
    /// Requests currently being served, peak value observed.
    peak_in_flight: Arc<AtomicUsize>,
    requests: Arc<AtomicUsize>,
    shutdown: Arc<AtomicBool>,
}

impl Server {
    fn start(behaviour: Behaviour, delay: Duration) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().unwrap().port();
        let peak_in_flight = Arc::new(AtomicUsize::new(0));
        let in_flight = Arc::new(AtomicUsize::new(0));
        let requests = Arc::new(AtomicUsize::new(0));
        let shutdown = Arc::new(AtomicBool::new(false));

        {
            let peak = Arc::clone(&peak_in_flight);
            let in_flight = Arc::clone(&in_flight);
            let requests = Arc::clone(&requests);
            let shutdown = Arc::clone(&shutdown);
            thread::spawn(move || {
                for stream in listener.incoming() {
                    if shutdown.load(Ordering::Relaxed) {
                        break;
                    }
                    let Ok(stream) = stream else { continue };
                    let peak = Arc::clone(&peak);
                    let in_flight = Arc::clone(&in_flight);
                    let requests = Arc::clone(&requests);
                    thread::spawn(move || {
                        let current = in_flight.fetch_add(1, Ordering::SeqCst) + 1;
                        peak.fetch_max(current, Ordering::SeqCst);
                        let seq = requests.fetch_add(1, Ordering::SeqCst);
                        handle(stream, behaviour, delay, seq);
                        in_flight.fetch_sub(1, Ordering::SeqCst);
                    });
                }
            });
        }

        Self {
            port,
            peak_in_flight,
            requests,
            shutdown,
        }
    }

    fn endpoint(&self) -> String {
        format!("http://127.0.0.1:{}/v1", self.port)
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }
}

fn handle(mut stream: TcpStream, behaviour: Behaviour, delay: Duration, seq: usize) {
    let mut reader = BufReader::new(stream.try_clone().expect("clone"));
    let mut content_length = 0usize;
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line).unwrap_or(0) == 0 {
            return;
        }
        if line == "\r\n" || line == "\n" {
            break;
        }
        if let Some(value) = line.to_lowercase().strip_prefix("content-length:") {
            content_length = value.trim().parse().unwrap_or(0);
        }
    }

    let mut body = vec![0u8; content_length];
    reader.read_exact(&mut body).ok();
    let body = String::from_utf8_lossy(&body).to_string();

    if delay > Duration::ZERO {
        thread::sleep(delay);
    }

    if let Behaviour::FailFirst(n) = behaviour {
        if seq < n {
            let payload = "{\"error\":\"boom\"}";
            let response = format!(
                "HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                payload.len(),
                payload
            );
            finish(&mut stream, &response);
            return;
        }
    }

    // The prompt carries lines shaped as `INDEX|TEXT`; echo them back translated.
    let translated: Vec<String> = body
        .split("\\n")
        .filter_map(|chunk| {
            let chunk = chunk.trim();
            let (index, text) = chunk.split_once('|')?;
            let index: usize = index.trim().parse().ok()?;
            // The last line carries the tail of the JSON body; cut it off.
            let text = text.split('"').next().unwrap_or("").trim();
            Some(format!("{index}|[pt] {text}"))
        })
        .collect();

    let content = translated.join("\\n");
    let payload = format!(
        "{{\"choices\":[{{\"message\":{{\"role\":\"assistant\",\"content\":\"{}\"}}}}]}}",
        content
    );
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        payload.len(),
        payload
    );
    finish(&mut stream, &response);
}

/// Writes the response and closes the write half gracefully — on Windows a
/// bare drop can RST the socket and lose the body.
fn finish(stream: &mut TcpStream, response: &str) {
    stream.write_all(response.as_bytes()).ok();
    stream.flush().ok();
    stream.shutdown(Shutdown::Write).ok();
    let mut sink = Vec::new();
    stream.read_to_end(&mut sink).ok();
}

fn entries(count: usize) -> Vec<(usize, String)> {
    (1..=count)
        .map(|index| (index, format!("line {index}")))
        .collect()
}

fn client(server: &Server) -> LlmClient {
    LlmClient::new(LlmConfig {
        endpoint: server.endpoint(),
        api_key: "test".into(),
        model: "test-model".into(),
        ..Default::default()
    })
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap()
}

#[test]
fn parallel_requests_stay_in_flight_together() {
    let server = Server::start(Behaviour::Ok, Duration::from_millis(120));
    let client = client(&server);
    let settings = TranslationSettings {
        batch_size: 10,
        parallel_requests: 4,
        max_retries: 1,
        ..Default::default()
    };

    let updates: Arc<Mutex<Vec<BatchUpdate>>> = Arc::new(Mutex::new(Vec::new()));
    let observer = {
        let updates = Arc::clone(&updates);
        Arc::new(move |update: BatchUpdate| updates.lock().unwrap().push(update))
    };

    let report = runtime().block_on(async {
        client
            .translate_all_batched(
                "translate",
                &entries(80),
                &settings,
                None,
                |_| {},
                |_| {},
                |_| {},
                |_| {},
                observer,
            )
            .await
            .expect("translation")
    });

    assert_eq!(report.translations.len(), 80, "every line comes back");
    assert!(!report.progress.is_partial);
    assert!(report.error_message.is_none());
    assert!(
        report
            .translations
            .iter()
            .all(|(_, text)| text.starts_with("[pt]")),
        "translations are applied"
    );

    let peak = server.peak_in_flight.load(Ordering::SeqCst);
    assert!(
        peak >= 3,
        "expected several concurrent requests, peak was {peak}"
    );

    let updates = updates.lock().unwrap();
    let completed: Vec<&BatchUpdate> = updates
        .iter()
        .filter(|u| u.status == BatchStatus::Completed)
        .collect();
    assert_eq!(completed.len(), 8, "one completion per batch");
    assert!(updates.iter().any(|u| u.status == BatchStatus::Pending));
    assert!(updates.iter().any(|u| u.status == BatchStatus::Active));
}

#[test]
fn a_slow_batch_does_not_block_the_other_slots() {
    // Two parallel slots, four batches: with wave scheduling this would take
    // two full rounds; with a proper queue every slot stays busy.
    let server = Server::start(Behaviour::Ok, Duration::from_millis(150));
    let client = client(&server);
    let settings = TranslationSettings {
        batch_size: 5,
        parallel_requests: 2,
        ..Default::default()
    };

    let start = std::time::Instant::now();
    let report = runtime().block_on(async {
        client
            .translate_all_batched(
                "translate",
                &entries(20),
                &settings,
                None,
                |_| {},
                |_| {},
                |_| {},
                |_| {},
                Arc::new(|_| {}),
            )
            .await
            .expect("translation")
    });
    let elapsed = start.elapsed();

    assert_eq!(report.translations.len(), 20);
    assert!(
        elapsed < Duration::from_millis(900),
        "four batches over two slots took {elapsed:?}"
    );
}

#[test]
fn failed_batches_are_retried_then_reported() {
    let server = Server::start(Behaviour::FailFirst(2), Duration::ZERO);
    let client = client(&server);
    let settings = TranslationSettings {
        batch_size: 10,
        parallel_requests: 2,
        max_retries: 3,
        continue_on_error: true,
        ..Default::default()
    };

    let retries = Arc::new(AtomicUsize::new(0));
    let retries_seen = Arc::clone(&retries);

    let report = runtime().block_on(async {
        client
            .translate_all_batched(
                "translate",
                &entries(20),
                &settings,
                None,
                |_| {},
                move |_| {
                    retries_seen.fetch_add(1, Ordering::SeqCst);
                },
                |_| {},
                |_| {},
                Arc::new(|_| {}),
            )
            .await
            .expect("translation")
    });

    assert_eq!(
        report.translations.len(),
        20,
        "retries recover both batches"
    );
    assert!(retries.load(Ordering::SeqCst) >= 2, "retries were reported");
    assert!(server.requests.load(Ordering::SeqCst) >= 4);
}

#[test]
fn cancellation_stops_the_run() {
    let server = Server::start(Behaviour::Ok, Duration::from_millis(200));
    let client = client(&server);
    let settings = TranslationSettings {
        batch_size: 5,
        parallel_requests: 2,
        ..Default::default()
    };

    let flag = Arc::new(AtomicBool::new(false));
    let cancel = Arc::clone(&flag);
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(120));
        cancel.store(true, Ordering::Relaxed);
    });

    let result = runtime().block_on(async {
        client
            .translate_all_batched(
                "translate",
                &entries(100),
                &settings,
                Some(flag),
                |_| {},
                |_| {},
                |_| {},
                |_| {},
                Arc::new(|_| {}),
            )
            .await
    });

    let err = result.expect_err("run should be cancelled");
    assert!(err.contains(TRANSLATION_CANCELLED_ERROR), "got: {err}");
}

#[test]
fn multi_line_entries_keep_real_line_breaks() {
    let server = Server::start(Behaviour::Ok, Duration::ZERO);
    let client = client(&server);

    let entries = vec![(1, "first line\nsecond line".to_string())];
    let result = runtime()
        .block_on(async { client.translate_subtitles("translate", &entries).await })
        .expect("translation");

    assert_eq!(result.len(), 1);
    let text = &result[0].1;
    assert!(
        text.contains('\n'),
        "line break should come back as a real newline, got {text:?}"
    );
    assert!(
        !text.contains(r"\N"),
        "no ASS escape should leak into the text, got {text:?}"
    );
}

#[test]
fn srt_output_keeps_real_line_breaks_end_to_end() {
    use translator::core::cancel::TranslationCancelState;
    use translator::core::subtitle::{SubtitleFile, SubtitleFormat};
    use translator::core::translate::{translate_subtitle_full, TranslationCallbacks};

    let server = Server::start(Behaviour::Ok, Duration::ZERO);
    let srt = "1\n00:00:01,000 --> 00:00:02,000\nfirst line\nsecond line\n";
    let file = SubtitleFile::parse(srt, SubtitleFormat::Srt).expect("parse");
    assert!(
        file.entries[0].text.contains('\n'),
        "source has a line break"
    );

    let config = LlmConfig {
        endpoint: server.endpoint(),
        api_key: "test".into(),
        model: "test-model".into(),
        ..Default::default()
    };

    let result = runtime().block_on(async {
        translate_subtitle_full(
            config,
            "translate".into(),
            file,
            TranslationSettings::default(),
            "file-1".into(),
            None,
            &TranslationCancelState::default(),
            TranslationCallbacks::default(),
        )
        .await
        .expect("translation")
    });

    let text = &result.file.entries[0].text;
    assert!(text.contains('\n'), "entry keeps the newline, got {text:?}");

    let serialized = result.file.serialize();
    assert!(
        !serialized.contains(r"\N"),
        "SRT output must not contain ASS escapes:\n{serialized}"
    );
}

#[test]
fn ass_output_keeps_the_ass_line_break_escape() {
    use translator::core::cancel::TranslationCancelState;
    use translator::core::subtitle::{SubtitleFile, SubtitleFormat};
    use translator::core::translate::{translate_subtitle_full, TranslationCallbacks};

    let server = Server::start(Behaviour::Ok, Duration::ZERO);
    let dialogue = concat!(
        "Dialogue: 0,0:00:01.00,0:00:02.00,Default,,0,0,0,,first line",
        r"\N",
        "second line"
    );
    let header =
        "[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text";
    let ass = format!("{header}\n{dialogue}\n");
    let file = SubtitleFile::parse(&ass, SubtitleFormat::Ass).expect("parse");

    let config = LlmConfig {
        endpoint: server.endpoint(),
        api_key: "test".into(),
        model: "test-model".into(),
        ..Default::default()
    };

    let result = runtime().block_on(async {
        translate_subtitle_full(
            config,
            "translate".into(),
            file,
            TranslationSettings::default(),
            "file-ass".into(),
            None,
            &TranslationCancelState::default(),
            TranslationCallbacks::default(),
        )
        .await
        .expect("translation")
    });

    let serialized = result.file.serialize();
    assert!(
        serialized.contains(r"\N"),
        "ASS keeps its own line break escape:\n{serialized}"
    );
}
