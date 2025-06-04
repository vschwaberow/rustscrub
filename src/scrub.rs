// SPDX-License-Identifier: MIT
// Project: rustscrub
// Description: A program to remove comments from source files.
// File: src/scrub.rs
// Author: Volker Schwaberow <volker@schwaberow.de>
// Copyright (c) 2025 Volker Schwaberow

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerboseCommentType {
    Line,
    Block,
}

#[derive(Debug, Clone)]
pub struct ChangeInfo {
    pub start_line: usize,
    pub end_line: usize,
    pub comment_type: VerboseCommentType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
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
pub struct StreamState {
    pub current_parse_state: State,
    pub raw_string_hash_count: usize,
    pub active_block_comment_start_line: Option<usize>,
    pub is_processing_full_line_comment: bool,
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

pub fn process_line_streaming(
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

