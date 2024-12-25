use emoji_commit_type::CommitType;
use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
};
use termion::input::TermRead;

pub fn git_parse_existing_message<R: Read>(reader: &mut R) -> Option<(CommitType, String)> {
    let first_line = reader.read_line().unwrap().unwrap();

    if first_line.is_empty() {
        return None;
    }

    let first_str = first_line.chars().next().unwrap().to_string();

    let commit_type =
        CommitType::iter_variants().find(|commit_type| first_str == commit_type.emoji());

    if commit_type == None {
        return None;
    }

    // Check that the rest of the commit message is empty (i.e. no body)
    if !git_message_is_empty(reader) {
        return None;
    }

    let emoji = commit_type.unwrap().emoji().to_string();
    let message = first_line.replace(&emoji, "").trim().to_string();
    Some((commit_type.unwrap(), message))
}

pub(crate) fn git_message_is_empty<R: Read>(reader: R) -> bool {
    for line in BufReader::new(reader).lines() {
        let line = line.expect("Failed to read line from git message file");

        if !line.starts_with('#') && !line.is_empty() {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Seek, Write};
    use tempfile::NamedTempFile;

    #[test]
    fn test_git_message_is_empty() {
        let test_cases: Vec<(&str, bool)> = vec![
            ("# Comment line\n\n# Another comment", true),
            ("yo", false),
            ("# Comment\nActual content", false),
            ("", true),
        ];

        for (input, expected) in test_cases {
            let mut file = NamedTempFile::new().unwrap();
            write!(file, "{}", input).unwrap();
            file.flush().unwrap();
            file.rewind().unwrap();
            assert_eq!(git_message_is_empty(&mut file), expected);
        }
    }

    #[test]
    fn test_git_parse_existing_message() {
        let test_cases: Vec<(&str, Option<(CommitType, &str)>)> = vec![
            ("🎉 Added a new feature", Some((CommitType::Feature, "Added a new feature"))),
            ("Invalid message", None),
            ("🐛 Fix critical bug", Some((CommitType::Bugfix, "Fix critical bug"))),
            ("🎉 Added a new feature\n\n# Comment", Some((CommitType::Feature, "Added a new feature"))),
            ("💥", Some((CommitType::Breaking, ""))),
        ];

        for (input, expected) in test_cases {
            let mut file = NamedTempFile::new().unwrap();
            writeln!(file, "{}", input).unwrap();
            file.flush().unwrap();
            file.rewind().unwrap();
            assert_eq!(
                git_parse_existing_message(&mut file),
                expected.map(|(t, m)| (t, m.to_string()))
            );
        }
    }
}
