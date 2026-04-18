use clap::Parser;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use rand::seq::SliceRandom;
use serde::Deserialize;
use std::fs;
use std::io::{stdout, Write};
use std::time::{Duration, Instant};

/// タイピングデータの構造体
#[derive(Deserialize, Debug, Clone)]
struct TypingWord {
    japanese: String,
    romaji: String,
}

/// コマンドライン引数の設定
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 制限時間（秒）
    #[arg(short, long, default_value_t = 60)]
    time: u64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 引数のパース
    let args = Args::parse();
    let time_limit = Duration::from_secs(args.time);

    // 単語データの読み込み
    let file_content = fs::read_to_string("words.json").expect("words.jsonが見つかりません。");
    let words: Vec<TypingWord> = serde_json::from_str(&file_content).expect("JSONのパースに失敗しました。");

    if words.is_empty() {
        println!("words.jsonにデータがありません。");
        return Ok(());
    }

    // ターミナルのセットアップ（Rawモードでリアルタイム入力を受け付ける）
    let mut stdout = stdout();
    enable_raw_mode()?;
    execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;

    let start_time = Instant::now();
    let mut rng = rand::thread_rng();
    let mut score = 0;
    let mut miss_count = 0;

    // ゲームループ
    'game: loop {
        // 残り時間のチェック
        let elapsed = start_time.elapsed();
        if elapsed >= time_limit {
            break 'game;
        }

        // ランダムに単語を選ぶ
        let current_word = words.choose(&mut rng).unwrap().clone();
        let mut input_index = 0;
        let target_chars: Vec<char> = current_word.romaji.chars().collect();

        // 1単語の入力ループ
        while input_index < target_chars.len() {
            let elapsed = start_time.elapsed();
            if elapsed >= time_limit {
                break 'game; // 単語の入力途中でも時間切れなら終了
            }
            let remaining = time_limit.saturating_sub(elapsed).as_secs();

            // 画面の描画
            execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
            execute!(
                stdout,
                Print(format!("残り時間: {}秒 | スコア: {}\r\n\n", remaining, score)),
                Print(format!("  {}\r\n", current_word.japanese)),
                Print("  ")
            )?;

            // 入力済みの文字を緑で表示
            execute!(stdout, SetForegroundColor(Color::Green))?;
            for i in 0..input_index {
                execute!(stdout, Print(target_chars[i]))?;
            }
            
            // 未入力の文字を白（デフォルト）で表示
            execute!(stdout, ResetColor)?;
            for i in input_index..target_chars.len() {
                execute!(stdout, Print(target_chars[i]))?;
            }
            execute!(stdout, Print("\r\n"))?;
            stdout.flush()?;

            // キー入力待ち（0.1秒単位でタイムアウトさせてループを回し、時間切れを検知する）
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key_event) = event::read()? {
                    // キーが押されたときのみ処理する
                    if key_event.kind == KeyEventKind::Press {
                        match key_event.code {
                            KeyCode::Char(c) => {
                                // 入力された文字が合っているか判定
                                if c == target_chars[input_index] {
                                    input_index += 1;
                                    score += 1;
                                } else {
                                    miss_count += 1;
                                }
                            }
                            KeyCode::Esc => {
                                // Escキーで強制終了
                                break 'game;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    // 後処理（ターミナルを元の状態に戻す）
    disable_raw_mode()?;
    execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0), ResetColor)?;

    // 結果発表
    println!("=== タイムアップ！ ===");
    println!("スコア (正しく打てた文字数): {}", score);
    println!("ミスタイプ数: {}", miss_count);
    let accuracy = if score + miss_count > 0 {
        (score as f64 / (score + miss_count) as f64) * 100.0
    } else {
        0.0
    };
    println!("正確率: {:.1}%", accuracy);

    Ok(())
}
