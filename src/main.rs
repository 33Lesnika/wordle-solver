use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;

#[derive(Parser)]
#[command(
    name = "wordle-solver",
    about = "Решатель Wordle: фильтрует слова по подсказкам.",
    long_about = "Решатель Wordle. Позволяет фильтровать слова по догадке и шаблону (pattern).\n\
    В интерактивном режиме поддерживаются команды: show, new, exit.\n\
    Формат pattern: строка из символов g (green), y (yellow), b (black), например: ybbgy."
)]
struct Args {
    #[arg(short, long, default_value = "wordle-La.txt")]
    dictionary: String,
    #[arg(short, long, requires = "pattern", help = "Догадка (слово, например: crate)")]
    guess: Option<String>,
    #[arg(
        short,
        long,
        requires = "guess",
        help = "Шаблон результата (pattern): строка из символов g (green), y (yellow), b (black).\n\
        Пример: ybbgy"
    )]
    pattern: Option<String>,

    #[arg(short, long, help = "Включить интерактивный режим")]
    interactive: bool,
}

fn load_dictionary<P: AsRef<Path>>(filename: P) -> io::Result<Vec<String>> {
    let file = File::open(filename)?;
    let reader = io::BufReader::new(file);
    let words = reader.lines().filter_map(Result::ok).collect();
    Ok(words)
}

fn check_greens(word_chars: &[char], guess_chars: &[char], pattern_chars: &[char], used_in_word: &mut [bool]) -> bool {
    for i in 0..word_chars.len() {
        if pattern_chars[i].eq_ignore_ascii_case(&'g') {
            if word_chars[i] != guess_chars[i] {
                return false;
            }
            used_in_word[i] = true;
        }
    }
    true
}

fn check_yellows(word_chars: &[char], guess_chars: &[char], pattern_chars: &[char], used_in_word: &mut [bool]) -> bool {
    for i in 0..word_chars.len() {
        if pattern_chars[i].eq_ignore_ascii_case(&'y') {
            if word_chars[i] == guess_chars[i] {
                return false;
            }
            let mut found = false;
            for j in 0..word_chars.len() {
                if !used_in_word[j] && word_chars[j] == guess_chars[i] && j != i {
                    used_in_word[j] = true;
                    found = true;
                    break;
                }
            }
            if !found {
                return false;
            }
        }
    }
    true
}

fn check_blacks(word_chars: &[char], guess_chars: &[char], pattern_chars: &[char], used_in_word: &[bool]) -> bool {
    for i in 0..word_chars.len() {
        if pattern_chars[i].eq_ignore_ascii_case(&'b') {
            for j in 0..word_chars.len() {
                if !used_in_word[j] && word_chars[j] == guess_chars[i] {
                    return false;
                }
            }
        }
    }
    true
}

fn matches_pattern(word: &str, guess: &str, pattern: &str) -> bool {
    let word_chars: Vec<char> = word.chars().collect();
    let guess_chars: Vec<char> = guess.chars().collect();
    let pattern_chars: Vec<char> = pattern.chars().collect();

    if word_chars.len() != guess_chars.len() || guess_chars.len() != pattern_chars.len() {
        return false;
    }

    let mut used_in_word = vec![false; word_chars.len()];

    if !check_greens(&word_chars, &guess_chars, &pattern_chars, &mut used_in_word) {
        return false;
    }
    if !check_yellows(&word_chars, &guess_chars, &pattern_chars, &mut used_in_word) {
        return false;
    }
    if !check_blacks(&word_chars, &guess_chars, &pattern_chars, &used_in_word) {
        return false;
    }

    true
}

fn main() -> io::Result<()> {
    let mut args = Args::parse();
    let dictionary = load_dictionary(&args.dictionary)?;

    if args.guess.is_none() && args.pattern.is_none() {
        args.interactive = true;
    }

    if args.interactive {
        println!(
            "Решатель Wordle: фильтрует слова по подсказкам\n\
            Введите вашу догадку и шаблон результата (pattern), который выдал Wordle\n\
            Команды:\n\
            show  — показать текущий список подходящих слов\n\
            new   — сбросить фильтр к исходному словарю\n\
            exit  — выйти из программы\n"
        );
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let mut filtered: Vec<String> = dictionary.clone();

        loop {
            print!("Введите guess (или команду show/new/exit): ");
            stdout.flush()?;
            let mut input = String::new();
            stdin.read_line(&mut input)?;
            let input = input.trim();

            if input.eq_ignore_ascii_case("exit") || input.is_empty() {
                break;
            } else if input.eq_ignore_ascii_case("show") {
                println!("Подходит {} слов:", filtered.len());
                for word in &filtered {
                    println!("{}", word);
                }
                continue;
            } else if input.eq_ignore_ascii_case("new") {
                filtered = dictionary.clone();
                println!("Список слов сброшен. Всего {} слов.", filtered.len());
                continue;
            }

            let guess = input;
            print!("Введите pattern (прим. ybbgy): ");
            stdout.flush()?;
            let mut pattern = String::new();
            stdin.read_line(&mut pattern)?;
            let pattern = pattern.trim();

            if pattern.is_empty() {
                break;
            }

            filtered = filtered
                .into_iter()
                .filter(|word| matches_pattern(word, guess, pattern))
                .collect();

            println!("Подходит {} слов.", filtered.len());
        }
    } else {
        let guess = args.guess.as_deref().expect("Не указан guess");
        let pattern = args.pattern.as_deref().expect("Не указан pattern");
        let filtered: Vec<_> = dictionary
            .iter()
            .filter(|word| matches_pattern(word, guess, pattern))
            .collect();

        println!("Подходит {} слов:", filtered.len());
        for word in filtered {
            println!("{}", word);
        }
    }

    Ok(())
}