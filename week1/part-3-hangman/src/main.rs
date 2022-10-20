// Simple Hangman Program
// User gets five incorrect guesses
// Word chosen randomly from words.txt
// Inspiration from: https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html
// This assignment will introduce you to some fundamental syntax in Rust:
// - variable declaration
// - string manipulation
// - conditional statements
// - loops
// - vectors
// - files
// - user input
// We've tried to limit/hide Rust's quirks since we'll discuss those details
// more in depth in the coming lectures.
extern crate rand;
use rand::Rng;
use std::fs;
use std::io;
use std::io::Write;
use std::iter;

const NUM_INCORRECT_GUESSES: u32 = 5;
const WORDS_PATH: &str = "words.txt";

fn pick_a_random_word() -> String {
    let file_string = fs::read_to_string(WORDS_PATH).expect("Unable to read file.");
    let words: Vec<&str> = file_string.split('\n').collect();
    let mut valid_words: Vec<&str> = vec![];
    for i in &words
    {
        if i.len() != 0
        {
            valid_words.push(i);
        }
    }
    assert_ne!(valid_words.len(),0);
    String::from(valid_words[rand::thread_rng().gen_range(0..valid_words.len())].trim())
}

fn vec_to_string(v: &Vec<char>) -> String {
    v.into_iter().collect()
}

fn main() {
    let secret_word = pick_a_random_word();
    // Note: given what you know about Rust so far, it's easier to pull characters out of a
    // vector than it is to pull them out of a string. You can get the ith character of
    // secret_word by doing secret_word_chars[i].
    let secret_word_chars: Vec<char> = secret_word.chars().collect();
    // Uncomment for debugging:
    println!("random word: {}", secret_word);

    // Your code here! :)
    println!("Welcome to CS110L Hangman!");
    let mut guess_times = 5;
    let mut word_so_far = iter::repeat('-').take(secret_word_chars.len()).collect();

    let mut guessed_letters = vec![];
    while guess_times > 0 {
        println!("The word so far is {}", vec_to_string(&word_so_far));
        println!(
            "You have guessed the following letters: {}",
            vec_to_string(&guessed_letters)
        );
        println!("You have {} guesses left", guess_times);
        print!("Please guess a letter: ");
        io::stdout().flush().expect("Error flushing stdout.");
        let mut guess = String::new();
        io::stdin()
            .read_line(&mut guess)
            .expect("Error reading line.");
        if guess.trim().len() != 1 {
            println!("Please input one letter, got {}", guess);
            continue;
        }
        let letter = guess.trim().chars().next().unwrap();
        guessed_letters.push(letter);

        let mut matched_letters = 0;
        let mut mismatched_letters = 0;
        for (i, &c) in secret_word_chars.iter().enumerate() {
            if c == letter {
                word_so_far[i] = letter;
                matched_letters += 1;
            }
            if word_so_far[i] == '-' {
                mismatched_letters += 1;
            }
        }
        if mismatched_letters == 0 {
            println!(
                "\nCongratulations you guessed the secret word: {}",
                vec_to_string(&word_so_far)
            );
            break;
        }
        if matched_letters == 0 {
            println!("Sorry, that letter is not in the word");
            guess_times -= 1;
            if guess_times == 0 {
                println!("\nSorry, you ran out of guesses!");
            }
        }
        println!("");
    }
}
