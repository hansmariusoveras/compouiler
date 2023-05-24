extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::{env, fs};
use pest::{Parser, iterators::Pair};

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct CSVParser;

fn parse_graph_definition(rule: Pair<Rule>) {
    println!("[DEFINE] {:} ", rule.as_str());
}

fn parse(rule: Pair<Rule>, depth: usize) {
    for record in rule.into_inner() {
        println!("{:}{:?}", "  ".repeat(depth), record.as_rule());
        match record.as_rule() {
            Rule::GraphDefinition => {
                parse_graph_definition(record);
            }
            Rule::EOI => {},
            Rule::Program => {},
            Rule::Statement => {
                parse(record, depth + 1);
            },
            Rule::Edges => {},
            Rule::EdgeDefinition => {},
            Rule::Assignment => {},
            Rule::Connect => {},
            Rule::ShortestPath => {},
            Rule::Value => {},
            Rule::NodeAccess => {},
            Rule::Edge =>{},
            Rule::Nodename => {},
            Rule::Graphname => {},
            Rule::WHITESPACE => {},
        
        }
    }  
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: noddy <filename>");
        return;
    }
    let filename = &args[1];
    let text = fs::read_to_string(filename).expect("Something went wrong reading the input file");
    let parse_tree = CSVParser::parse(Rule::Program, &text).expect("Syntax error.").next().unwrap();
    parse(parse_tree, 0)
} 
