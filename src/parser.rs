use clap::Parser;
use std::fs;
use std::io;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use crate::{Graph, Node};

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    /// Filename
    #[arg(short, long)]
    file: String,

    /// Number of threads
    #[arg(short = 'j', long, default_value_t = 1)]
    threads: u8,

    /// Targets to run
    targets: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Invalid makefile syntax at line {line}: {message}")]
    Syntax { line: usize, message: String },
    #[error("Circular dependency detected")]
    CircularDependency,
    #[error("Target '{target}' not found")]
    TargetNotFound { target: String },
}

impl Graph {
    pub fn parse_makefile(&mut self, filename: &str) -> Result<(), ParseError> {
        let file = fs::File::open(filename)?;
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().collect::<Result<Vec<_>, _>>()?;

        let mut first_target = None;

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();

            if line.is_empty() || line.starts_with('#') {
                i += 1;
                continue;
            }

            if line.contains(':') && !line.starts_with('\t') {
                let (target, lines_consumed) = self.parse_rule(&lines, i)?;

                if first_target.is_none() {
                    first_target = Some(target);
                }

                i += 1 + lines_consumed;
                continue;
            }

            i += 1;
        }

        match first_target {
            Some(target) => self.default_target = target,
            None => (),
        }

        Ok(())
    }

    fn parse_rule(
        &mut self,
        lines: &[String],
        line_idx: usize,
    ) -> Result<(String, usize), ParseError> {
        let rule_line = &lines[line_idx].trim();

        let colon_pos = rule_line.find(':').ok_or_else(|| ParseError::Syntax {
            line: line_idx + 1,
            message: "Rule missing colon".to_string(),
        })?;

        let target_name = rule_line[..colon_pos].trim().to_string();
        if target_name.is_empty() {
            return Err(ParseError::Syntax {
                line: line_idx + 1,
                message: "Rule missing target".to_string(),
            });
        }

        let deps_substr = rule_line[colon_pos + 1..].trim();
        let deps: Vec<String> = if deps_substr.is_empty() {
            Vec::new()
        } else {
            deps_substr
                .split_whitespace()
                .map(|s| s.to_string())
                .collect()
        };

        let mut commands = Vec::new();
        let mut lines_consumed = 0;
        for idx in (line_idx + 1)..lines.len() {
            let line = &lines[idx];

            if line.starts_with('\t') || line.starts_with(' ') {
                let command = if line.starts_with('\t') {
                    line[1..].to_string()
                } else {
                    line.trim_start().to_string()
                };

                commands.push(command);
                lines_consumed += 1;
            } else if line.trim().is_empty() {
                lines_consumed += 1;
            } else {
                break;
            }
        }

        let mut node = Node::new(target_name.clone());
        node.dependencies = deps;
        node.commands = commands;
        node.output = Some(PathBuf::from(&target_name));
        let mut nodes = self.nodes.try_write().unwrap();
        nodes.insert(target_name.clone(), node);

        Ok((target_name, lines_consumed))
    }

    pub fn debug_print(&self) {
        let nodes = self.nodes.try_read().unwrap();
        println!("Graph Debug:");
        println!("Default target: {}", self.default_target);
        println!("Nodes:");
        for (target, node) in nodes.iter() {
            println!(
                "  {}: deps={:?}, commands={:?}",
                target, node.dependencies, node.commands
            );
        }
    }
}
