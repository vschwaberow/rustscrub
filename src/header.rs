// SPDX-License-Identifier: MIT
// Project: rustscrub
// Description: A program to remove comments from source files.
// File: src/header.rs
// Author: Volker Schwaberow <volker@schwaberow.de>
// Copyright (c) 2025 Volker Schwaberow

use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn detect_header(file_path: &Path) -> Result<(usize, String), String> {
    let file = File::open(file_path)
        .map_err(|e| format!("Failed to open file for header detection: {}", e))?;

    let reader = BufReader::new(file);
    let mut lines = Vec::new();
    let mut line_count = 0;
    let mut in_header = true;

    let mut saw_code = false;
    let mut saw_normal_comment = false;
    let mut empty_line_count = 0;

    const MAX_PREVIEW_LINES: usize = 10;
    const MAX_HEADER_SIZE: usize = 50;

    for line in reader.lines() {
        let line = line.map_err(|e| format!("Failed to read line during header detection: {}", e))?;
        line_count += 1;

        if line_count <= MAX_PREVIEW_LINES {
            lines.push(line.clone());
        }

        let trimmed = line.trim();

        if trimmed.is_empty() {
            empty_line_count += 1;
            if empty_line_count > 2 && saw_normal_comment {
                in_header = false;
                break;
            }
            continue;
        } else {
            empty_line_count = 0;
        }

        if trimmed.starts_with("#![") {
            saw_normal_comment = true;
            continue;
        }

        if trimmed.starts_with("//!") || trimmed.starts_with("///") {
            saw_normal_comment = true;
            continue;
        }

        if trimmed.starts_with("//") || trimmed.starts_with("/*") {
            saw_normal_comment = true;
            continue;
        }

        if trimmed.starts_with("use ") || trimmed.starts_with("mod ") ||
           trimmed.starts_with("pub ") || trimmed.starts_with("fn ") ||
           trimmed.starts_with("struct ") || trimmed.starts_with("enum ") ||
           trimmed.starts_with("impl ") || trimmed.starts_with("trait ") {
            saw_code = true;
            in_header = false;
            break;
        }

        if line_count > 3 && saw_normal_comment {
            in_header = false;
            break;
        }

        if line_count > MAX_HEADER_SIZE {
            in_header = false;
            break;
        }
    }

    let header_lines = if saw_code && !in_header {
        line_count - 1
    } else if saw_normal_comment {
        line_count
    } else {
        0
    };

    let preview = if !lines.is_empty() {
        let preview_text = lines.join("\n");
        if lines.len() < line_count {
            format!("{}\n... ({} more lines)", preview_text, line_count - lines.len())
        } else {
            preview_text
        }
    } else {
        "".to_string()
    };

    Ok((header_lines, preview))
}

pub fn ask_yes_no_question(question: &str) -> bool {
    use std::io::{stdin, stdout};

    print!("{} [y/N]: ", question);
    stdout().flush().unwrap_or(());

    let mut response = String::new();

    if stdin().read_line(&mut response).is_err() {
        return false;
    }

    let response = response.trim().to_lowercase();
    response == "y" || response == "yes"
}

