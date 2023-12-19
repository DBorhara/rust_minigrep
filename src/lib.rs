extern crate regex;
use regex::{Captures, Regex};
use std::error::Error;
use std::{env, fs};

pub struct Config {
    pub query: String,
    pub file_path: String,
    pub ignore_case: bool,
    pub replacement: Option<String>,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("Not enough arguments");
        }

        let query = args[1].clone();
        let file_path = args[2].clone();

        let ignore_case = env::var("IGNORE_CASE").is_ok();

        let replacement = if args.len() > 3 {
            Some(args[3].clone())
        } else {
            None
        };

        return Ok(Config {
            query,
            file_path,
            ignore_case,
            replacement,
        });
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.file_path)?;
    if let Some(replacement) = config.replacement {
        let modified_conent = search_and_replace(&config.query, &replacement, &contents)?;
        println!("{}", modified_conent)
    }
    let results = if config.ignore_case {
        search_case_insensitive(&config.query, &contents)
    } else {
        search(&config.query, &contents)
    };

    for line in results {
        println!("{}", line)
    }
    Ok(())
}

pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let mut results = Vec::new();

    for line in contents.lines() {
        if line.contains(query) {
            results.push(line)
        }
    }

    return results;
}

pub fn search_case_insensitive<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let query = query.to_lowercase();
    let mut results = Vec::new();

    for line in contents.lines() {
        if line.to_lowercase().contains(&query) {
            results.push(line);
        }
    }
    return results;
}

pub fn search_and_replace<'a>(
    query: &str,
    replacment: &str,
    contents: &'a str,
) -> Result<String, regex::Error> {
    let word_boundary = format!(r"\b(?i){}\b", regex::escape(query));
    let re = Regex::new(&word_boundary)?;

    Ok(re
        .replace_all(contents, |caps: &Captures| {
            match_case(caps.get(0).unwrap().as_str(), replacment)
        })
        .into_owned())
}

fn match_case(original: &str, replacment: &str) -> String {
    if original == original.to_uppercase() {
        replacment.to_uppercase()
    } else if original == original.to_lowercase() {
        replacment.to_lowercase()
    } else if original.chars().next().unwrap().is_uppercase() {
        replacment
            .chars()
            .next()
            .unwrap()
            .to_uppercase()
            .to_string()
            + &replacment.chars().skip(1).collect::<String>()
    } else {
        replacment.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_sensitive() {
        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Duct tape.";
        assert_eq!(vec!["safe, fast, productive."], search(query, contents));
    }
    #[test]
    fn case_insensitive() {
        let query = "rUsT";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";

        assert_eq!(
            vec!["Rust:", "Trust me."],
            search_case_insensitive(query, contents)
        );
    }

    #[test]
    fn replace() {
        let query = "three";
        let replacement = "3";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";

        let expected = "\
Rust:
safe, fast, productive.
Pick 3.
Trust me.";

        assert_eq!(
            Ok(expected.to_string()),
            search_and_replace(query, replacement, contents),
        );
    }
}
