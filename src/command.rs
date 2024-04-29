#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Arg(String),
    Args(Vec<Expr>),
    ColumnFilter(Box<Expr>),
}

#[derive(Debug, PartialEq)]
enum Token {
    String(String),
    Ident(String),
    Equal,
    Comma,
}

pub fn parse(input: &str) -> Result<Vec<Expr>, String> {
    let tokens = tokenizer(input);
    parse_tokens(tokens)
}

fn tokenizer(input: &str) -> Vec<Token> {
    let mut result = Vec::new();

    input.split_whitespace().for_each(|s| {
        let mut draft_token = String::from("");

        for c in s.chars() {
            match c {
                '=' => {
                    result.extend(vec![Token::Ident(draft_token.clone()), Token::Equal]);
                    draft_token.clear();
                }
                ',' => {
                    result.extend(vec![Token::String(draft_token.clone()), Token::Comma]);
                    draft_token.clear();
                }
                _ => draft_token.push(c),
            }
        }
        if !draft_token.is_empty() {
            result.push(Token::String(draft_token.clone()));
        }
    });

    result
}

fn string_tokenizer(input: &str) -> Vec<Token> {
    let mut result = Vec::new();
    let mut input = input.chars().peekable();

    while let Some(c) = input.next() {
        match c {
            ',' => result.push(Token::Comma),
            _ => {
                let mut s = c.to_string();
                while let Some(&c) = input.peek() {
                    match c {
                        ',' => break,
                        _ => {
                            s.push(c);
                            input.next();
                        }
                    }
                }
                result.push(Token::String(s));
            }
        }
    }

    result
}

fn parse_tokens(tokens: Vec<Token>) -> Result<Vec<Expr>, String> {
    let mut exprs = Vec::new();
    let mut tokens = tokens.iter().peekable();

    while let Some(token) = tokens.next() {
        match token {
            Token::Ident(s) => {
                if s == "column" {
                    if tokens.next() != Some(&Token::Equal) {
                        return Err("Invalid token".to_string());
                    }

                    let mut args = Vec::new();
                    while let Some(token) = tokens.peek() {
                        match token {
                            Token::String(_) => {
                                if let Token::String(value) = tokens.next().unwrap() {
                                    args.push(Expr::Arg(value.clone()));
                                }
                            }
                            Token::Comma => {
                                tokens.next();
                            }
                            _ => {
                                break;
                            }
                        }
                    }
                    exprs.push(Expr::ColumnFilter(Box::new(Expr::Args(args))));
                } else {
                    return Err(format!("Not found {}", s));
                }
            }
            _ => return Err("Invalid token not match".to_string()),
        }
    }

    Ok(exprs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod tokenizer {
        use super::*;

        #[test]
        fn inputting() {
            let expr: Vec<Token> = vec![Token::String("h".to_string())];
            assert_eq!(expr, tokenizer("h"));
        }

        #[test]
        fn inputting_with_equal() {
            let expr = vec![Token::Ident("h".to_string()), Token::Equal];
            assert_eq!(expr, tokenizer("h="));
        }

        #[test]
        fn single() {
            let expr = vec![
                Token::Ident("hoge".to_string()),
                Token::Equal,
                Token::String("foo".to_string()),
            ];
            assert_eq!(expr, tokenizer("hoge=foo"))
        }

        #[test]
        fn multi_clause() {
            let expr = vec![
                Token::Ident("hoge".to_string()),
                Token::Equal,
                Token::String("foo".to_string()),
                Token::Ident("piyo".to_string()),
                Token::Equal,
                Token::String("bar".to_string()),
            ];
            assert_eq!(expr, tokenizer("hoge=foo piyo=bar"))
        }

        #[test]
        fn multi_args() {
            let expr = vec![
                Token::Ident("hoge".to_string()),
                Token::Equal,
                Token::String("foo".to_string()),
                Token::Comma,
                Token::String("bar".to_string()),
            ];
            assert_eq!(expr, tokenizer("hoge=foo,bar"))
        }
    }

    #[cfg(test)]
    mod parse {
        use super::*;

        #[test]
        fn inputting() {
            let tokens = vec![Token::String("h".to_string())];
            assert_eq!(
                Err("Invalid token not match".to_string()),
                parse_tokens(tokens)
            );
        }

        #[test]
        fn inputting_with_equal() {
            let tokens = vec![Token::Ident("h".to_string()), Token::Equal];
            assert_eq!(Err("Not found h".to_string()), parse_tokens(tokens));
        }

        #[test]
        fn single() {
            let tokens = vec![
                Token::Ident("column".to_string()),
                Token::Equal,
                Token::String("foo".to_string()),
            ];

            assert_eq!(
                Ok(vec![Expr::ColumnFilter(Box::new(Expr::Args(vec![
                    Expr::Arg("foo".to_string())
                ])))]),
                parse_tokens(tokens)
            );
        }

        #[test]
        fn double() {
            let tokens = vec![
                Token::Ident("column".to_string()),
                Token::Equal,
                Token::String("foo".to_string()),
                Token::Ident("column".to_string()),
                Token::Equal,
                Token::String("bar".to_string()),
            ];

            assert_eq!(
                Ok(vec![
                    Expr::ColumnFilter(Box::new(Expr::Args(vec![Expr::Arg("foo".to_string())]))),
                    Expr::ColumnFilter(Box::new(Expr::Args(vec![Expr::Arg("bar".to_string())]))),
                ]),
                parse_tokens(tokens)
            );
        }

        #[test]
        fn multi_args() {
            let expect = vec![Expr::ColumnFilter(Box::new(Expr::Args(vec![
                Expr::Arg("foo".to_string()),
                Expr::Arg("bar".to_string()),
            ])))];

            let tokens = vec![
                Token::Ident("column".to_string()),
                Token::Equal,
                Token::String("foo".to_string()),
                Token::Comma,
                Token::String("bar".to_string()),
            ];

            assert_eq!(Ok(expect), parse_tokens(tokens));
        }

        #[test]
        fn miss_equal() {
            let tokens = vec![
                Token::Ident("column".to_string()),
                Token::String("foo".to_string()),
            ];

            assert_eq!(Err("Invalid token".to_string()), parse_tokens(tokens));
        }
    }
}
