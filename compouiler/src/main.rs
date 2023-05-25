extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::{env, fs, collections::HashMap};
use pest::{Parser, iterators::Pair};
use fdg_img::{self, style::{TextStyle, BLACK, Color, IntoFont, text_anchor::{Pos, HPos, VPos}}, Settings};
use fdg_sim::{ForceGraph, ForceGraphHelper, petgraph::Undirected};

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct CSVParser;

#[derive(PartialEq, Eq)]
#[derive(Debug)]
pub enum NodeKind {
    Root,
    GraphDefinition,
    EdgeDefinition,
    Node,
    Assignment,
    Number,
    Connection,
}

struct Node {
    children: Vec<Node>,
    kind: NodeKind,
    load: i32
}

struct NodeTable {
    symbols: HashMap<String, i32>,
}

struct Context {
    graph: Option<String>,
    graphs: HashMap<String, NodeTable>,
    variables: Vec<i32>,
    edges: Vec<(i32, i32)>,
}


fn new_empty_node(kind: NodeKind) -> Node {
    Node { children: Vec::new(), kind, load: 0 }
}

fn parse_graph_reference(rule: Pair<Rule>, node: &mut Node, context: &mut Context) {
    let name = rule.as_str().to_string();
    if node.kind != NodeKind::GraphDefinition && !context.graphs.contains_key(&name) {
        panic!("Error: Referring to undeclared graph.");
    }
    if !context.graphs.contains_key(&name) {
        context.graphs.insert(name.clone(), NodeTable { symbols: HashMap::new()});
    }
    context.graph = Some(name);
}

fn parse_node_reference(rule: Pair<Rule>, node: &mut Node, context: &mut Context) {
    if context.graph.is_none() {
        panic!("Graph reference outside of graph definition.");
    }
    let graph = context.graph.as_ref().unwrap();
    let mut node_node = new_empty_node(NodeKind::Node);
    assert!( rule.as_rule() == Rule::Nodename || rule.as_rule() == Rule::Graphname);
    let name = rule.as_str();
    let pointer: i32;
    let table = context.graphs.get_mut(graph).unwrap();
    if node.kind != NodeKind::EdgeDefinition && !table.symbols.contains_key(name) {
        panic!("Error: Referring to undeclared node.");
    }
    if !table.symbols.contains_key(name) {
        pointer = context.variables.len() as i32;
        table.symbols.insert(name.to_string(), pointer);
        context.variables.push(0);
    }
    else {
        pointer = table.symbols[name];
    }
    node_node.load = pointer;
    node.children.push(node_node);

}

fn parse_edge_definition(rule: Pair<Rule>, node: &mut Node, context: &mut Context) {
    assert! (node.kind == NodeKind::GraphDefinition);
    let mut edge_definition_node = new_empty_node(NodeKind::EdgeDefinition);
    parse(rule, &mut edge_definition_node, context);
    context.edges.push((edge_definition_node.children[0].load, edge_definition_node.children[1].load));
    node.children.push(edge_definition_node);
}

fn parse_graph_definition(rule: Pair<Rule>, node: &mut Node, context: &mut Context) {
    let mut graph_definition_node = new_empty_node(NodeKind::GraphDefinition);
    parse(rule, &mut graph_definition_node, context);

    assert!(node.kind == NodeKind::Root);
    node.children.push(graph_definition_node);
}

fn parse(rule: Pair<Rule>, node: &mut Node, context: &mut Context) {
    for record in rule.into_inner() {
        match record.as_rule() {
            Rule::GraphDefinition => {
                parse_graph_definition(record, node, context);
            }
            Rule::EOI => {},
            Rule::Program => {
                parse(record, node, context);
            },
            Rule::Statement => {
                parse(record, node, context);
            },
            Rule::Edges => {
                parse(record, node, context);
            },
            Rule::EdgeDefinition => {
                parse_edge_definition(record, node, context);
            },
            Rule::Assignment => {
                let mut assignment_node = new_empty_node(NodeKind::Assignment);
                parse(record, &mut assignment_node, context);
                let left = assignment_node.children[0].load;
                let right: i32;
                match assignment_node.children[1].kind {
                    NodeKind::Number => {
                        right = assignment_node.children[1].load;
                    },
                    NodeKind::Node => {
                        right = context.variables[assignment_node.children[1].load as usize];
                    },
                    _ => {
                        panic!("Invalid assignment");
                    }
                }
                context.variables[left as usize] = right;
                node.children.push(assignment_node);
            },
            Rule::Connect => {
                let mut connection_node = new_empty_node(NodeKind::Connection);
                parse(record, &mut connection_node, context);
                let left = connection_node.children[0].load;
                let right = connection_node.children[1].load;
                context.edges.push((left, right));


            },
            Rule::ShortestPath => {},
            Rule::Value => {
                parse(record, node, context);
            },
            Rule::Number => {
                let mut number_node = new_empty_node(NodeKind::Number);
                number_node.load = record.as_str().parse::<i32>().unwrap();
                node.children.push(number_node);
            },
            Rule::NodeAccess => {
                parse(record, node, context);
            },
            Rule::Edge =>{
                parse(record, node, context);
            },
            Rule::Nodename => {
                parse_node_reference(record, node, context);
            },
            Rule::Graphname => {
                parse_graph_reference(record, node, context);
            },
            Rule::WHITESPACE => {},
        
        }
    }  
}

fn print_tables(context: &Context) {
    for (name, table) in &context.graphs {
        println!("Graph: {}", name);
        for (name, pointer) in &table.symbols {
            println!(" {} {}: {}", pointer, name, &context.variables[*pointer as usize]);
        }
    }
}

fn print_edges(context: &Context) {
    println!("Edges:");
    for (from, to) in &context.edges {
        println!(" {} -> {}", from, to);
    }
}

fn print_ast(node: &mut Node, depth: i32) {
    let mut indent = String::new();
    for _ in 0..depth {
        indent.push_str("  ");
    }
    println!("{} [{}] {:?}", indent, node.load, node.kind);
    for child in &mut node.children {
        print_ast(child, depth + 1);
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
    let mut root: Node = new_empty_node(NodeKind::Root);
    let mut context = Context {
        graph: None,
        graphs: HashMap::new(),
        variables: Vec::new(),
        edges: Vec::new()
    };
    parse(parse_tree, &mut root, &mut context);
    print_tables(&context);
    print_edges(&context);
    print_ast(&mut root, 0);

    let mut graph:ForceGraph<(), ()> = ForceGraph::default();
    let mut nodes = Vec::new();
    for i in 0..context.variables.len() {
        nodes.push(graph.add_force_node(i.to_string(), ()));
    }
    for edge in &context.edges {
        graph.add_edge(nodes[edge.0 as usize], nodes[edge.1 as usize], ());
    }
    let text_style = Some(TextStyle {
        font: ("sans-serif", 20).into_font(),
        color: BLACK.to_backend_color(),
        pos: Pos {
            h_pos: HPos::Left,
            v_pos: VPos::Center,
        },
    });
    let svg = fdg_img::gen_image(graph, Some(Settings{text_style,..Default::default()}));
    // save the svg on disk (or send it to an svg renderer)
    fs::write("ring.svg", svg.unwrap().as_bytes()).unwrap();
}