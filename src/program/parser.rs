use serde::{Serialize, Deserialize};
use nom::{
    branch::alt, bytes::complete::{tag, take_while}, character::complete::{  digit1, multispace0}, combinator::{map, map_res, opt},  sequence::{ preceded, tuple}, IResult
};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufRead};
use std::str::FromStr;


#[derive(Serialize, Deserialize, Debug)]
struct Task {
    id: String,
    name: String,
    description: String,
    prompt: String,
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    operator: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Input {
    name: String,
    value: Value,
    required: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct Output {
    type_: String,
    key: String,
    value: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Value {
    type_: String,
    key: String,
}

#[derive(Debug, Clone)]
enum Token {
    Identifier(String),
    StringLiteral(String),
    Symbol(char),
    Parenthesis(char), // '(' or ')'
}

#[derive(Debug, Deserialize, Serialize)]
struct StepsFile {
    steps: Vec<Step>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Step {
    source: String,
    target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    fallback: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    condition: Option<Condition>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Condition {
    input: ConditionInput,
    expression: Expression,
    expected: String,
    target_if_not: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ConditionInput {
    #[serde(rename = "type")]
    input_type: String,
    key: String,
}

#[derive(Debug, Deserialize, Serialize)]
enum Expression {
    Equal,
    NotEqual,
    Contains,
    NotContains,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    HaveSimilar,
}

impl FromStr for Expression {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "==" => Ok(Expression::Equal),
            "!=" => Ok(Expression::NotEqual),
            "contains" => Ok(Expression::Contains),
            "!contains" => Ok(Expression::NotContains),
            ">" => Ok(Expression::GreaterThan),
            "<" => Ok(Expression::LessThan),
            ">=" => Ok(Expression::GreaterThanOrEqual),
            "<=" => Ok(Expression::LessThanOrEqual),
            "~=" => Ok(Expression::HaveSimilar),
            _ => Err(()),
        }
    }
}

// Used for Tasks 
fn lexer_tasks(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut iter = input.chars().peekable();

    while let Some(c) = iter.next() {
        match c {
            '=' | '?' | ',' => tokens.push(Token::Symbol(c)),
            '(' | ')' => tokens.push(Token::Parenthesis(c)),
            '"' => {
                let mut value = String::new();
                while let Some(ch) = iter.next() {
                    if ch == '"' {
                        break;
                    } else {
                        value.push(ch);
                    }
                }
                tokens.push(Token::StringLiteral(value));
            },
            _ if c.is_alphanumeric() || c == '_' => {
                let mut ident = c.to_string();
                while let Some(&next) = iter.peek() {
                    if next.is_alphanumeric() || next == '_' {
                        ident.push(iter.next().unwrap());
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Identifier(ident));
            },
            _ if c.is_whitespace() => continue,
            _ => (), // Ignore unknown or irrelevant characters
        }
    }

    tokens
}
// Used for Tasks input 
fn parse_input(input_str: &str, task: &mut Task) {
    let parts: Vec<_> = input_str.split('.').map(str::trim).collect();
    if let Some(input_name) = parts.get(0) {
        let method_parts: Vec<_> = parts.get(1).unwrap().split('(').map(str::trim).collect();
        let method = method_parts[0];
        let key = method_parts.get(1).unwrap_or(&"").trim_end_matches(')').to_string();
        let required = !input_str.ends_with('?');
        
        task.inputs.push(Input {
            name: input_name.to_string(),
            value: Value {
                type_: method.to_string(),
                key,
            },
            required,
        });
    }
}
// Used for Tasks output
fn parse_output(output_str: &str, task: &mut Task) {
    // Split the function call at the first '(' to isolate the type and arguments
    let type_end_index = output_str.find('(').unwrap_or(output_str.len());
    let output_type = &output_str[..type_end_index];

    // Extract arguments within the parentheses
    let args_start = output_str.find('(').map(|i| i + 1).unwrap_or(output_str.len());
    let args_end = output_str.rfind(')').unwrap_or(output_str.len());
    let args_str = &output_str[args_start..args_end];

    // Split the argument string to extract key and value
    let arg_parts: Vec<_> = args_str.splitn(2, '(').map(str::trim).collect();
    if arg_parts.len() == 2 {
        let key = arg_parts[0];
        let value_with_extra_paren = arg_parts[1];
        let value = value_with_extra_paren.trim_end_matches(')');

        task.outputs.push(Output {
            type_: output_type.to_string(),
            key: key.to_string(),
            value: value.to_string(),
        });
    } else {
        // Handle error or unexpected format
        println!("Unexpected format in output string: {}", output_str);
    }
}

// Used for Tasks
fn parse_tasks(tokens: Vec<Token>) -> Vec<Task> {
    let mut tasks = Vec::new();
    let mut current_task = Task {
        id: "".to_string(),
        name: "".to_string(),
        description: "".to_string(),
        prompt: "".to_string(),
        inputs: Vec::new(),
        outputs: Vec::new(),
        operator: "".to_string(),
    };

    let mut i = 0;
    while i < tokens.len() {
        match tokens.get(i) {
            Some(Token::Identifier(ident)) => {
                match ident.as_str() {
                    "id" | "name" | "description" | "prompt" | "operator" => {
                        if i + 1 < tokens.len() {
                            if let Some(Token::StringLiteral(value)) = tokens.get(i + 1) {
                                match ident.as_str() {
                                    "id" => current_task.id = value.clone(),
                                    "name" => current_task.name = value.clone(),
                                    "description" => current_task.description = value.clone(),
                                    "prompt" => current_task.prompt = value.clone(),
                                    "operator" => current_task.operator = value.clone(),
                                    _ => {}
                                }
                                i += 2; // Skip past the identifier and its value
                                continue;
                            }
                        }
                        i += 1; // Move past the identifier even if no valid value was found
                    },
                    "input_output" => {
                        i += 1; // Move past "input_output"
                        while i < tokens.len() && matches!(tokens.get(i), Some(Token::Identifier(_))) {
                            if let Some(Token::Identifier(section)) = tokens.get(i) {
                                i += 1; // Move to either the input or output list
                                while i < tokens.len() && matches!(tokens.get(i), Some(Token::StringLiteral(_))) {
                                    if let Some(Token::StringLiteral(io_str)) = tokens.get(i) {
                                        if section == "input" {
                                            parse_input(io_str, &mut current_task);
                                        } else if section == "output" {
                                            parse_output(io_str, &mut current_task);
                                        }
                                    }
                                    i += 1; // Move past the current input or output string
                                    if i < tokens.len() && matches!(tokens.get(i), Some(Token::Symbol(','))) {
                                        i += 1; // Skip comma between items
                                    }
                                }
                            }
                        }
                    },
                   
                    _ => i += 1,
                }
            },
            Some(Token::Symbol(',')) => i += 1, // Handle commas within the main structure
            _ => i += 1, // Default case to increment index safely
        }
    }

    tasks.push(current_task);
    tasks
}

fn parse_step(input: &str) -> IResult<&str, Step> {
    let (input, source) = parse_identifier(input)?;
    let (input, _) = tag("->")(input)?;
    let (input, target) = parse_identifier(input)?;
    let (input, _) = multispace0(input)?;
    let (input, condition) = opt(preceded(tag("! if("), parse_condition))(input)?;
    let (input, fallback) = opt(preceded(tag("! if("), parse_fallback))(input)?;

    Ok((
        input,
        Step {
            source: source.to_string(),
            target: target.to_string(),
            condition,
            fallback,
        },
    ))
}


fn is_alphanumeric_or_dot(c: char) -> bool {
    c.is_alphanumeric() || c == '.'
}

fn parse_identifier(input: &str) -> IResult<&str, &str> {
    take_while(is_alphanumeric_or_dot)(input)
}

fn parse_expression(input: &str) -> IResult<&str, Expression> {
    map_res(
        alt((
            tag("=="),
            tag("!="),
            tag("contains"),
            tag("!contains"),
            tag(">"),
            tag("<"),
            tag(">="),
            tag("<="),
            tag("~="),
        )),
        FromStr::from_str,
    )(input)
}

fn parse_condition(input: &str) -> IResult<&str, Condition> {
    let (input, (key, _, input_type, _, expression, _, expected, _, target_if_not)) = tuple((
        parse_identifier,
        tag("."),
        parse_identifier,
        multispace0,
        parse_expression,
        multispace0,
        alt((digit1, parse_identifier)),
        tag(") else "),
        parse_identifier,
    ))(input)?;

    Ok((
        input,
        Condition {
            input: ConditionInput {
                input_type: input_type.to_string(),
                key: key.to_string(),
            },
            expression,
            expected: expected.to_string(),
            target_if_not: target_if_not.to_string(),
        },
    ))
}

fn main() {
    let input = r#"
        {
            id: "A",
            name: "Web Search Query",
            description: "Collect info",
            prompt: "Query?",
            operator: "generation"
            input_output: [
                input:{
                    "query.input()",
                    "history.get_all(“history”)?"
			    },
	            output:{
                    "write(web_search_query(__result)",
                    "push(history(__result))"
			}
            ],
        }
    "#;

    let tokens = lexer_tasks(input);
    let tasks = parse_tasks(tokens);

    let output_json5 = serde_json::to_string_pretty(&tasks).unwrap();
    println!("{}", output_json5);
}