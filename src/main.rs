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

fn matches_pattern(word: &str, guess: &str, pattern: &str) -> bool {
    let w: Vec<char> = word.chars().collect();
    let g: Vec<char> = guess.chars().collect();
    let p: Vec<char> = pattern.chars().collect();

    if w.len() != g.len() || g.len() != p.len() {
        return false;
    }

    let mut used = vec![false; w.len()];
    let mut guess_used = vec![false; g.len()];

    for i in 0..w.len() {
        if p[i] == 'g' || p[i] == 'G' {
            if w[i] != g[i] {
                return false;
            }
            used[i] = true;
            guess_used[i] = true;
        }
    }

    for i in 0..w.len() {
        if p[i] == 'y' || p[i] == 'Y' {
            if w[i] == g[i] {
                return false;
            }
            let mut found = false;
            for j in 0..w.len() {
                if !used[j] && w[j] == g[i] {
                    used[j] = true;
                    found = true;
                    break;
                }
            }
            if !found {
                return false;
            }
            guess_used[i] = true;
        }
    }

    for i in 0..w.len() {
        if p[i] == 'b' || p[i] == 'B' {
            for j in 0..w.len() {
                if !used[j] && w[j] == g[i] {
                    return false;
                }
            }
        }
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