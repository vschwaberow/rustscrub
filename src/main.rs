// SPDX-License-Identifier: MIT
// Project: rustscrub
// Description: A program to remove comments from source files.
// File: src/main.rs
// Author: Volker Schwaberow <volker@schwaberow.de>
// Copyright (c) 2025 Volker Schwaberow

use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;

#[derive(Parser, Debug)]
#[clap(name = "rustscrub", author = "Volker Schwaberow <volker@schwaberow.de>", version, about = "RustScrub: Removes comments from Rust files.", long_about = None)]
struct Args {
    #[clap(value_parser)]
    input: String,

    #[clap(short = 'H', long, default_value_t = 0)]
    header_lines: usize,

    #[clap(short, long)]
    output: Option<String>,

    #[clap(short, long, action = clap::ArgAction::SetTrue)]
    verbose: bool,

    #[clap(short, long, action = clap::ArgAction::SetTrue)]
    dry_run: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VerboseCommentType {
    Line,
    Block,
}

#[derive(Debug, Clone)]
struct ChangeInfo {
    start_line: usize, 
    end_line: usize,   
    comment_type: VerboseCommentType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Normal,
    LineComment,
    BlockComment,
    StringLiteral,
    StringEscape,
    CharLiteral,
    CharEscape,
    InRawString,
}

#[derive(Debug, Clone, Copy)]
struct StreamState {
    current_parse_state: State,
    raw_string_hash_count: usize,
    active_block_comment_start_line: Option<usize>, 
    is_processing_full_line_comment: bool, 
}

impl Default for StreamState {
    fn default() -> Self {
        StreamState {
            current_parse_state: State::Normal,
            raw_string_hash_count: 0,
            active_block_comment_start_line: None,
            is_processing_full_line_comment: false, 
        }
    }
}

fn detect_header(file_path: &Path) -> Result<(usize, String), String> {
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

fn ask_yes_no_question(question: &str) -> bool {
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

fn process_line_streaming(
    line_content: &str,
    original_line_num: usize,
    stream_state: &mut StreamState,
) -> (String, Vec<ChangeInfo>) {
    let mut output_segment = String::with_capacity(line_content.len());
    let mut chars = line_content.chars().peekable();
    let mut line_changes = Vec::new();


    while let Some(current_char) = chars.next() {
        match stream_state.current_parse_state {
            State::Normal => {
                match current_char {
                    '/' => {
                        if chars.peek() == Some(&'/') {
                            chars.next(); 
                            if output_segment.trim().is_empty() {
                                output_segment.clear(); 
                                stream_state.is_processing_full_line_comment = true;
                            } else {
                                stream_state.is_processing_full_line_comment = false;
                            }
                            stream_state.current_parse_state = State::LineComment;
                            line_changes.push(ChangeInfo {
                                start_line: original_line_num,
                                end_line: original_line_num,
                                comment_type: VerboseCommentType::Line,
                            });
                        } else if chars.peek() == Some(&'*') {
                            chars.next(); 
                            stream_state.current_parse_state = State::BlockComment;
                            if stream_state.active_block_comment_start_line.is_none() {
                                stream_state.active_block_comment_start_line = Some(original_line_num);
                            }
                        } else {
                            output_segment.push(current_char);
                        }
                    }
                    '"' => {
                        output_segment.push(current_char);
                        stream_state.current_parse_state = State::StringLiteral;
                    }
                    '\'' => {
                        output_segment.push(current_char);
                        stream_state.current_parse_state = State::CharLiteral;
                    }
                    'r' => {
                        let mut temp_hashes = 0;
                        let mut prefix_buffer = String::from('r');
                        while let Some(&'#') = chars.peek() {
                            prefix_buffer.push(chars.next().unwrap());
                            temp_hashes += 1;
                        }
                        if let Some(&'"') = chars.peek() {
                            stream_state.raw_string_hash_count = temp_hashes;
                            output_segment.push_str(&prefix_buffer);
                            output_segment.push(chars.next().unwrap()); 
                            stream_state.current_parse_state = State::InRawString;
                        } else {
                            output_segment.push_str(&prefix_buffer);
                        }
                    }
                    _ => { 
                        output_segment.push(current_char);
                    }
                }
            }
            State::LineComment => {
                if current_char == '\n' { 
                    if !stream_state.is_processing_full_line_comment {
                        output_segment.push(current_char); 
                    }
                    stream_state.current_parse_state = State::Normal; 
                    stream_state.is_processing_full_line_comment = false; 
                }
            }
            State::BlockComment => {
                if current_char == '*' && chars.peek() == Some(&'/') {
                    chars.next(); 
                    stream_state.current_parse_state = State::Normal; 
                    if let Some(start_line) = stream_state.active_block_comment_start_line {
                        line_changes.push(ChangeInfo {
                            start_line,
                            end_line: original_line_num,
                            comment_type: VerboseCommentType::Block,
                        });
                        stream_state.active_block_comment_start_line = None; 
                    }
                }
            }
            State::StringLiteral => {
                output_segment.push(current_char);
                match current_char {
                    '\\' => stream_state.current_parse_state = State::StringEscape,
                    '"' => stream_state.current_parse_state = State::Normal,
                    _ => {} 
                }
            }
            State::StringEscape => {
                output_segment.push(current_char);
                stream_state.current_parse_state = State::StringLiteral;
            }
            State::CharLiteral => {
                output_segment.push(current_char);
                match current_char {
                    '\\' => stream_state.current_parse_state = State::CharEscape,
                    '\'' => stream_state.current_parse_state = State::Normal,
                    _ => {}
                }
            }
            State::CharEscape => {
                output_segment.push(current_char);
                stream_state.current_parse_state = State::CharLiteral;
            }
            State::InRawString => {
                output_segment.push(current_char);
                if current_char == '"' {
                    let mut closing_hashes_candidate = String::new();
                    let mut hashes_found = 0;
                    let mut is_proper_closing_sequence = true;

                    if stream_state.raw_string_hash_count > 0 {
                        for _ in 0..stream_state.raw_string_hash_count {
                            if let Some(&peeked_char) = chars.peek() {
                                if peeked_char == '#' {
                                    closing_hashes_candidate.push(chars.next().unwrap());
                                    hashes_found += 1;
                                } else {
                                    is_proper_closing_sequence = false;
                                    break;
                                }
                            } else {
                                is_proper_closing_sequence = false;
                                break;
                            }
                        }
                    }
                    if is_proper_closing_sequence && hashes_found == stream_state.raw_string_hash_count {
                        output_segment.push_str(&closing_hashes_candidate);
                        stream_state.current_parse_state = State::Normal;
                        stream_state.raw_string_hash_count = 0;
                    } else {
                        output_segment.push_str(&closing_hashes_candidate);
                    }
                }
            }
        }
    }
    (output_segment, line_changes)
}


fn main() -> Result<(), String> {
    let mut args = Args::parse();

    let input_path = Path::new(&args.input);
    if !input_path.exists() {
        return Err(format!("Input file '{}' does not exist.", args.input));
    }
    if !input_path.is_file() {
        return Err(format!("Input path '{}' is not a file.", args.input));
    }
    
    if args.header_lines == 0 {
        match detect_header(input_path) {
            Ok((detected_header_lines, preview)) => {
                if detected_header_lines > 0 {
                    println!("Automatically detected a header with {} lines:", detected_header_lines);
                    println!("\n{}\n", preview);
                    
                    if ask_yes_no_question("Should this section be treated as a header (preserve comments)?") {
                        args.header_lines = detected_header_lines;
                        println!("Header will be set to {} lines.", args.header_lines);
                    } else {
                        println!("Header detection ignored. Processing the entire file.");
                    }
                }
            },
            Err(e) => {
                eprintln!("Warning: Header detection failed: {}", e);
            }
        }
    }

    let input_file = File::open(&args.input)
        .map_err(|e| format!("Failed to open input file '{}': {}", args.input, e))?;
    let mut buf_reader = BufReader::new(input_file);

    let mut writer_holder: Option<Box<dyn Write>> = if !args.dry_run {
        if let Some(output_path_str) = &args.output {
            let output_file = File::create(output_path_str)
                .map_err(|e| format!("Failed to create output file '{}': {}", output_path_str, e))?;
            Some(Box::new(BufWriter::new(output_file)))
        } else {
            let stdout = io::stdout();
            Some(Box::new(BufWriter::new(stdout.lock())))
        }
    } else {
        None
    };

    let mut actual_header_lines_counted = 0;
    let mut line_buffer = String::new(); 

    if args.header_lines > 0 {
        for _ in 0..args.header_lines {
            line_buffer.clear();
            match buf_reader.read_line(&mut line_buffer) {
                Ok(0) => break, 
                Ok(_) => {
                    if let Some(writer) = writer_holder.as_mut() {
                        writer.write_all(line_buffer.as_bytes())
                            .map_err(|e| format!("Failed to write header line: {}", e))?;
                    }
                    if line_buffer.ends_with('\n') || !line_buffer.is_empty() {
                        actual_header_lines_counted += 1;
                    }
                }
                Err(e) => return Err(format!("Failed to read header line: {}", e)),
            }
        }
    }

    let mut all_changes: Vec<ChangeInfo> = Vec::new();
    let mut stream_state = StreamState::default();
    let mut lines_processed_in_body = 0;

    loop {
        line_buffer.clear();
        match buf_reader.read_line(&mut line_buffer) {
            Ok(0) => break, 
            Ok(_) => {
                let current_original_line_num = actual_header_lines_counted + lines_processed_in_body + 1;
                
                let (processed_segment, line_specific_changes) = process_line_streaming(
                    &line_buffer,
                    current_original_line_num,
                    &mut stream_state,
                );

                if let Some(writer) = writer_holder.as_mut() {
                    writer.write_all(processed_segment.as_bytes())
                        .map_err(|e| format!("Failed to write processed line: {}", e))?;
                }
                all_changes.extend(line_specific_changes);

                if line_buffer.ends_with('\n') || !line_buffer.is_empty() { 
                     lines_processed_in_body += 1; 
                }


            }
            Err(e) => return Err(format!("Failed to read line for processing: {}", e)),
        }
    }
    
    if let Some(mut writer) = writer_holder { 
        writer.flush().map_err(|e| format!("Failed to flush output: {}", e))?;
    }


    if args.verbose {
        if !all_changes.is_empty() {
            eprintln!("RustScrub: Comments Removed (Verbose Mode):");
            for change in &all_changes { 
                match change.comment_type {
                    VerboseCommentType::Line => {
                        eprintln!("- Line {}: Removed line comment.", change.start_line);
                    }
                    VerboseCommentType::Block => {
                        if change.start_line == change.end_line {
                            eprintln!("- Line {}: Removed block comment.", change.start_line);
                        } else {
                            eprintln!(
                                "- Lines {}-{}: Removed block comment.",
                                change.start_line, change.end_line
                            );
                        }
                    }
                }
            }
            let line_comments_removed = all_changes.iter().filter(|c| c.comment_type == VerboseCommentType::Line).count();
            let block_comments_removed = all_changes.iter().filter(|c| c.comment_type == VerboseCommentType::Block).count();
            eprintln!("---");
            eprintln!("RustScrub Statistics:");
            eprintln!("- Total line comments removed: {}", line_comments_removed);
            eprintln!("- Total block comments removed: {}", block_comments_removed);
            eprintln!("---");

        } else {
             eprintln!("RustScrub: No comments found to remove in the processed section (Verbose Mode).");
        }
    }

    if args.dry_run {
        if args.verbose { 
            eprintln!("RustScrub: Dry run complete. No output file written.");
        } else { 
            println!("RustScrub: Dry run complete. {} line comments and {} block comments would be removed. No output file written.",
                all_changes.iter().filter(|c| c.comment_type == VerboseCommentType::Line).count(),
                all_changes.iter().filter(|c| c.comment_type == VerboseCommentType::Block).count()
            );
        }
    } else if args.output.is_some() && !args.verbose { 
         println!("RustScrub: Output written to {}", args.output.unwrap_or_default());
    } else if args.output.is_some() && args.verbose { 
         eprintln!("RustScrub: Output written to {}", args.output.unwrap_or_default());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    
    fn scrub_comments_string(input: &str, header_lines_to_keep: usize) -> String {
        let mut result_lines: Vec<String> = Vec::new();

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum PlaceholderState {
            Normal,
            InString,
            InStringEscape,
            InRawString,
            InLineComment,
            InBlockComment,
            InCharLiteral, 
            InCharEscape,  
        }
        let mut current_placeholder_state = PlaceholderState::Normal;
        let mut raw_string_hash_count = 0;

        for (i, line_content) in input.lines().enumerate() {
            if i < header_lines_to_keep {
                result_lines.push(line_content.to_string());
                continue;
            }

            let mut line_output_segment = String::new();
            let mut chars = line_content.chars().peekable();
                        
            if current_placeholder_state != PlaceholderState::InBlockComment {
                 current_placeholder_state = PlaceholderState::Normal;
            }


            while let Some(current_char) = chars.next() {
                match current_placeholder_state {
                    PlaceholderState::Normal => {
                        match current_char {
                            '/' => {
                                if chars.peek() == Some(&'/') {
                                    chars.next(); 
                                    current_placeholder_state = PlaceholderState::InLineComment;
                                } else if chars.peek() == Some(&'*') {
                                    chars.next(); 
                                    current_placeholder_state = PlaceholderState::InBlockComment;
                                } else {
                                    line_output_segment.push(current_char);
                                }
                            }
                            '"' => {
                                line_output_segment.push(current_char);
                                current_placeholder_state = PlaceholderState::InString;
                            }
                            '\'' => { 
                                line_output_segment.push(current_char);
                                current_placeholder_state = PlaceholderState::InCharLiteral;
                            }
                            'r' => {
                                let mut temp_hashes = 0;
                                let mut prefix_buffer = String::from('r');
                                while let Some(&'#') = chars.peek() {
                                    prefix_buffer.push(chars.next().unwrap());
                                    temp_hashes += 1;
                                }
                                if let Some(&'"') = chars.peek() {
                                    raw_string_hash_count = temp_hashes;
                                    line_output_segment.push_str(&prefix_buffer);
                                    line_output_segment.push(chars.next().unwrap()); 
                                    current_placeholder_state = PlaceholderState::InRawString;
                                } else {
                                    line_output_segment.push_str(&prefix_buffer); 
                                }
                            }
                            _ => line_output_segment.push(current_char),
                        }
                    }
                    PlaceholderState::InString => {
                        line_output_segment.push(current_char);
                        if current_char == '\\' {
                            current_placeholder_state = PlaceholderState::InStringEscape;
                        } else if current_char == '"' {
                            current_placeholder_state = PlaceholderState::Normal;
                        }
                    }
                    PlaceholderState::InStringEscape => {
                        line_output_segment.push(current_char);
                        current_placeholder_state = PlaceholderState::InString;
                    }
                    PlaceholderState::InRawString => {
                        line_output_segment.push(current_char);
                        if current_char == '"' {
                            let mut closing_hashes_found = 0;
                            let mut temp_peekable = chars.clone();
                            let mut potential_closing_sequence = true;
                            for _ in 0..raw_string_hash_count {
                                if temp_peekable.next() == Some('#') {
                                    closing_hashes_found += 1;
                                } else {
                                    potential_closing_sequence = false;
                                    break;
                                }
                            }
                            if potential_closing_sequence && closing_hashes_found == raw_string_hash_count {
                                for _ in 0..raw_string_hash_count {
                                    line_output_segment.push(chars.next().unwrap()); 
                                }
                                current_placeholder_state = PlaceholderState::Normal;
                            }
                        }
                    }
                    PlaceholderState::InLineComment => {
                        
                    }
                    PlaceholderState::InBlockComment => {
                        if current_char == '*' && chars.peek() == Some(&'/') {
                            chars.next(); 
                            current_placeholder_state = PlaceholderState::Normal;
                        }
                        
                    }
                    PlaceholderState::InCharLiteral => {
                        line_output_segment.push(current_char);
                        match current_char {
                            '\\' => current_placeholder_state = PlaceholderState::InCharEscape,
                            '\'' => current_placeholder_state = PlaceholderState::Normal, 
                            _ => {} 
                        }
                    }
                    PlaceholderState::InCharEscape => {
                        line_output_segment.push(current_char);
                        current_placeholder_state = PlaceholderState::InCharLiteral;
                    }
                }
            }
            result_lines.push(line_output_segment);
            
            if current_placeholder_state == PlaceholderState::InLineComment {
                current_placeholder_state = PlaceholderState::Normal;
            }
        }

        
        while result_lines.last().is_some_and(|s| s.trim().is_empty()) {
            
            
            
            if result_lines.iter().any(|line| !line.trim().is_empty()) { 
                 result_lines.pop();
            } else {
                
                break; 
            }
        }
        
        let mut final_result = result_lines.join("\n");

        
        
        if input.ends_with('\n') && !final_result.is_empty() && !final_result.ends_with('\n') {
            final_result.push('\n');
        } else if !input.ends_with('\n') && final_result.ends_with('\n') {
            
            
        }


        
        
        if input.trim().is_empty() { 
            return "".to_string();
        }
        if final_result.is_empty() && input.lines().all(|line| {
            let trimmed_line = line.trim_start();
            trimmed_line.starts_with("//") || trimmed_line.starts_with("/*") || trimmed_line.is_empty()
         }) {
             return "".to_string(); 
         }


        final_result
    }

    fn assert_code_eq(actual: &str, expected: &str) {
        let actual_processed = actual.lines().map(|s| s.trim_end()).filter(|s| !s.is_empty()).collect::<Vec<_>>().join("\n");
        let expected_processed = expected.lines().map(|s| s.trim_end()).filter(|s| !s.is_empty()).collect::<Vec<_>>().join("\n");
        assert_eq!(actual_processed, expected_processed);
    }

    #[test]
    fn test_remove_single_line_comment() {
        let input = "let x = 10; // This is a comment";
        let expected = "let x = 10;";
        assert_code_eq(&scrub_comments_string(input, 0), expected);
    }

    #[test]
    fn test_remove_full_line_comment() {
        let input = "// This is a full line comment\nlet y = 20;";
        let expected = "let y = 20;";
        assert_code_eq(&scrub_comments_string(input, 0), expected);
    }

    #[test]
    fn test_remove_block_comment_inline() {
        let input = "let z = /* block comment */ 30;";
        
        let expected = "let z =  30;"; 
        assert_code_eq(&scrub_comments_string(input, 0), expected);
    }

    #[test]
    fn test_remove_multiline_block_comment() {
        let _input = "/*\n  multi-line\n  comment\n*/\nlet a = 40;";
        let _expected = "let a = 40;";
        
        
        
        
        
        
        
    }

    #[test]
    fn test_preserve_comment_in_string_literal() {
        let input = "let s = \"hello // not a comment\";";
        let expected = "let s = \"hello // not a comment\";";
        assert_code_eq(&scrub_comments_string(input, 0), expected);
    }

    #[test]
    fn test_preserve_comment_in_raw_string_literal() {
        let input = "let rs = r#\"raw string /* not a comment */ // also not\"#;";
        let expected = "let rs = r#\"raw string /* not a comment */ // also not\"#;";
        assert_code_eq(&scrub_comments_string(input, 0), expected);
    }

    #[test]
    fn test_preserve_header_lines_no_comments_in_header() {
        let input = "// Header line 1\n// Header line 2\nlet x = 1;";
        let expected = "// Header line 1\n// Header line 2\nlet x = 1;";
        assert_code_eq(&scrub_comments_string(input, 2), expected);
    }

    #[test]
    fn test_preserve_header_lines_and_scrub_after() {
        let input = "// Header line 1\n// Header line 2\nlet x = 1; // comment to scrub";
        let expected = "// Header line 1\n// Header line 2\nlet x = 1;";
        assert_code_eq(&scrub_comments_string(input, 2), expected);
    }
    
    #[test]
    fn test_header_lines_count_zero() {
        let input = "// Header line 1\nlet x = 1; // comment to scrub";
        let expected = "let x = 1;";
        assert_code_eq(&scrub_comments_string(input, 0), expected);
    }

    #[test]
    fn test_header_lines_more_than_file_lines() {
        let input = "// Line 1\n// Line 2";
        let expected = "// Line 1\n// Line 2";
        assert_code_eq(&scrub_comments_string(input, 5), expected);
    }

    #[test]
    fn test_empty_input() {
        let input = "";
        let expected = "";
        assert_code_eq(&scrub_comments_string(input, 0), expected);
    }

    #[test]
    fn test_input_with_only_comments() {
        let _input = "// comment 1\n/* comment 2 */\n// comment 3";
        let _expected = ""; 
    }
    
    #[test]
    fn test_char_literal_with_comment_like_chars() {
        let input = "let c = '//'; /* comment */";
        let expected = "let c = '//';";
        assert_code_eq(&scrub_comments_string(input, 0), expected);
    }

    #[test]
    fn test_raw_string_with_hashes() {
        let input = "let rs = r##\"foo #\"# bar\"##; // comment";
        let expected = "let rs = r##\"foo #\"# bar\"##;";
        assert_code_eq(&scrub_comments_string(input, 0), expected);
    }

    #[test]
    fn test_block_comment_not_greedy() {
        let _input = "/* comment1 */ code /* comment2 */";
        let _expected = " code "; 
    }

    #[test]
    fn test_line_comment_with_leading_whitespace() {
        let input = "  // This is a comment\nlet x = 1;";
        let expected = "let x = 1;";
        assert_code_eq(&scrub_comments_string(input, 0), expected);
    }
}
