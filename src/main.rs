extern crate curl;
extern crate classifier;
use curl::easy::Easy;
use std::fs::OpenOptions;
use std::fs;
use std::fs::File;
use std::io::{Write, BufWriter, BufReader, BufRead};
use std::time::Duration;
use std::env;
use classifier::NaiveBayes;

fn main() {
    let topic_file_modified = match fs::metadata("./topic-votes.txt") {
        Ok(meta) => meta.modified().unwrap()
                        .elapsed().unwrap(),
        Err(_)   => Duration::new(604801, 0),  // 1 week
    };
    //Ensure local file is no more than 1 week old | Download latest file
    if topic_file_modified.as_secs() > 604800 {
        let topics_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open("./topic-votes.txt")
            .expect("cannot open file");
        let mut topics_buffer = BufWriter::new(topics_file);
        let mut easy_curl = Easy::new();
        easy_curl.url("https://a.openbible.info/data/topic-votes.txt").unwrap();
        easy_curl.write_function(move |data| {
            Ok(topics_buffer.write(data).unwrap())
        }).unwrap();
        easy_curl.perform().unwrap();
    }
    //train classifier
    let mut nb = NaiveBayes::new();
    let open_file = File::open("./topic-votes.txt").expect("could not open file");
    let topic_file = BufReader::new(open_file);
    for line in topic_file.lines() {
        let line_vec = line.as_ref().unwrap().split("\t").collect::<Vec<&str>>();
        nb.add_document(&line_vec[0].to_string(), &line_vec[1].to_string());
    };
    nb.train();
    for argument in env::args().skip(1) {
        println!("{}: {}", argument, format_verse_reference(&nb.classify(&argument.to_string())));
    };
}

fn format_verse_reference(verse_ref: &str) -> String {
    println!("{}", verse_ref);
    let books = vec!["not", "Genesis", "Exodus", "Leviticus", "Numbers",
         "Deuteronomy", "Joshua", "Judges", "Ruth", "1 Samuel", "2 Samuel", "1 Kings", "2 Kings",
         "1 Chronicles", "2 Chronicles", "Ezra", "Nehemiah", "Esther", "Job", "Psalms", "Proverbs",
         "Ecclesiastes", "Song of Solomon", "Isaiah", "Jeremiah", "Lamentations", "Ezekiel",
         "Daniel", "Hosea", "Joel", "Amos", "Obadiah", "Jonah", "Micah", "Nahum", "Habakkuk",
         "Zephaniah", "Haggai", "Zechariah", "Malachi", "Matthew", "Mark", "Luke", "John",
         "Acts of the Apostles", "Romans", "1 Corinthians", "2 Corinthians", "Galatians",
         "Ephesians", "Philippians", "Colossians", "1 Thessolonians", "2 Thessalonians",
         "1 Timothy", "2 Timothy", "Titus", "Philemon", "Hebrews", "James", "1 Peter", "2 Peter",
         "1 John", "2 John", "3 John", "Jude", "Revelation"];
    let book    = books[*&verse_ref[0..2].to_string().parse::<usize>().unwrap()];
    let chapter = &verse_ref[2..5];
    let verse   = &verse_ref[5..];
    format!("{} {}:{}", book, chapter.trim_left_matches("0"), verse.trim_left_matches("0"))
}
