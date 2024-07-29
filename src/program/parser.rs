use super::atomics::{Edge,Condition, Expression,InputValue,
    InputValueType,Task,Operator,Output,OutputType,Input};

use nom::{
    branch::alt, bytes::complete::{tag, take_while}, 
    character::complete::{  digit1, multispace0}, 
    combinator::{map, map_res, opt},  
    sequence::{ preceded, tuple}, IResult
};
use std::str::FromStr;


#[derive(Debug, Clone)]
pub enum Token {
    Identifier(String),
    StringLiteral(String),
    Symbol(char),
    Parenthesis(char), // '(' or ')'
    CurlyBrace(char), // '{' or '}'
    Other(char),
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

// ## TASKS PARSING 
pub fn lexer_tasks(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut iter = input.chars().peekable();
    let mut context_stack = Vec::new(); // Stack to track contexts like arrays

    enum ParseState {
        ExpectingKey,
        ExpectingValue,
    }

    let mut state = ParseState::ExpectingKey;

    while let Some(c) = iter.next() {
        match c {
            '[' => {
                // Entering an array context
                context_stack.push('[');
                tokens.push(Token::Symbol(c));
            },
            ']' => {
                // Exiting an array context
                if context_stack.pop().is_some() {
                    tokens.push(Token::Symbol(c));
                }
            },
            ',' => {
                tokens.push(Token::Symbol(c));
                // Only transition state if not within an array context
                if context_stack.is_empty() {
                    state = ParseState::ExpectingKey;
                }
            },
            ':' => {
                tokens.push(Token::Symbol(c));
                // Switch state after a colon, as it separates keys and values
                if context_stack.is_empty() { // Ensure we're not in an array
                    state = match state {
                        ParseState::ExpectingKey => ParseState::ExpectingValue,
                        ParseState::ExpectingValue => ParseState::ExpectingKey,
                    };
                }
            },
            '"' => {
                let mut value = String::new();
                while let Some(ch) = iter.next() {
                    if ch == '"' {
                        break;
                    } else {
                        value.push(ch);
                    }
                }
                if value == "input" || value == "output"{
                    tokens.push(Token::Identifier(value));
                } 
                else{
                tokens.push(match state {
                    ParseState::ExpectingKey => Token::Identifier(value),
                    ParseState::ExpectingValue => Token::StringLiteral(value),
                });
            }
            },
            _ if c.is_whitespace() => continue,
            _ => tokens.push(Token::Other(c)), // Handle other characters appropriately
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
            value:InputValue {
                key: key.to_string(),
                value_type: {
                    match method {
                        "input" => InputValueType::Input,
                        "read" => InputValueType::Read,
                        "pop" => InputValueType::Pop,
                        "peek" => InputValueType::Peek,
                        "get_all" => InputValueType::GetAll,
                        "size" => InputValueType::Size,
                        "string" => InputValueType::String,
                        _ => InputValueType::Input,
                    }
                },
                index: Option::None,
                search_query: Option::None,
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
            output_type:{
                match output_type {
                    "insert" => OutputType::Insert,
                    "write" => OutputType::Write,
                    "push" => OutputType::Push,
                    _ => OutputType::Write,
                }
            },
            key: key.to_string(),
            value: value.to_string(),
        });
    } else {
        // Handle error or unexpected format
        println!("Unexpected format in output string: {}", output_str);
    }
}

// Used for Tasks
pub fn parse_tasks(tokens: Vec<Token>) -> Task {
    let mut current_task = Task {
        id: "".to_string(),
        name: "".to_string(),
        description: "".to_string(),
        prompt: "".to_string(),
        inputs: Vec::new(),
        outputs: Vec::new(),
        operator: Operator::Generation,
    };
  
    let mut i = 0;
    while i < tokens.len() {
        match tokens.get(i) {
            Some(Token::Identifier(ident)) => {
                match ident.as_str() {
                    "id" | "name" | "description" | "prompt" | "operator" => {
                        if i + 1 < tokens.len() {
                            if let Some(Token::StringLiteral(value)) = tokens.get(i + 2) {
                                match ident.as_str() {
                                    "id" => current_task.id = value.clone(),
                                    "name" => current_task.name = value.clone(),
                                    "description" => current_task.description = value.clone(),
                                    "prompt" => current_task.prompt = value.clone(),
                                    "operator" => {
                                        match value.as_str() {
                                            "generation" => current_task.operator = Operator::Generation,
                                            "function_calling" => current_task.operator = Operator::FunctionCalling,
                                            "check" => current_task.operator = Operator::Check,
                                            "search" => current_task.operator = Operator::Search,
                                            "sample" => current_task.operator = Operator::Sample,
                                            "end" => current_task.operator = Operator::End,
                                            _ => {}
                                        }
                                    },
                                    _ => {}
                                }
                                i += 2; // Skip past the identifier and its value
                                continue;
                            }
                        }
                        i += 1; // Move past the identifier even if no valid value was found
                    },
"input_output" => {
    i += 2; // Move past "input_output" and the following ":"
    if matches!(tokens.get(i), Some(Token::Other('{'))) {
        i += 1; // Skip the opening curly brace
        while i < tokens.len() {
            match tokens.get(i) {
                Some(Token::CurlyBrace('}')) => {
                    i += 1; // Move past the closing curly brace
                    break; // Exit the loop as we've found the closing curly brace
                },
                Some(Token::Identifier(section)) => {
                    i += 2; // Move past the section identifier and the following ":"
                    if matches!(tokens.get(i), Some(Token::Symbol('['))) {
                        i += 1; // Skip the opening bracket
                        while i < tokens.len() && !matches!(tokens.get(i), Some(Token::Symbol(']'))) {
                            match tokens.get(i) {
                                Some(Token::StringLiteral(io_str)) | Some(Token::Identifier(io_str)) => {
                                    if section == "input" {
                                        parse_input(io_str, &mut current_task);
                                    } else if section == "output" {
                                        parse_output(io_str, &mut current_task);
                                    }
                                },
                                _ => {}
                            }
                            i += 1; // Move to the next token, could be a comma or the closing bracket
                        }
                        // No need to increment i here as it should now be on the closing bracket
                    }
                },
                _ => i += 1, // Default increment for other tokens
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

    current_task
}
// ## STEPS PARSING
pub fn parse_step(input: &str) -> IResult<&str, Edge> {
    let (input, source) = parse_identifier(input)?;
    let (input, _) = tag("->")(input)?;
    let (input, target) = parse_identifier(input)?;
    let (input, _) = multispace0(input)?;
    let (input, condition) = opt(preceded(tag("! if("), parse_condition))(input)?;
    let (input, fallback) = opt(preceded(tag("! if("), parse_fallback))(input)?;

    Ok((
        input,
        Edge {
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

fn parse_fallback(input: &str) -> IResult<&str, String> {
    let (input, _) = map(tag("fallback"), |_| ())(input)?;
    let (input, _) = map(tag(") else "), |_| ())(input)?;
    let (input, fallback_target) = parse_identifier(input)?;

    Ok((input, fallback_target.to_string()))
}

fn parse_condition(input: &str) -> IResult<&str, Condition> {
    let (input, (key, _, _input_type, _, expression, _, expected, _, target_if_not)) = tuple((
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
            input: InputValue {
                key: key.to_string(),
                value_type: InputValueType::Input, // steps does it have dif input?
                index: Option::None,
                search_query: Option::None,

            },
            expression,
            expected: expected.to_string(),
            target_if_not: target_if_not.to_string(),
        },
    ))
}
