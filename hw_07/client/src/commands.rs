use std::str::FromStr;

use regex::Regex;

use crate::errors::FromStrError;

#[derive(PartialEq)]
pub enum CommandType {
    File(String),
    Image(String),
    Text(String),
    Quit,
    Username(String),
    Color((u8, u8, u8)),
}

impl FromStr for CommandType {
    type Err = FromStrError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        // parse input from user by using regex. There are few options:
        // - .file <filename>
        // - .image <filename>
        // - .username <new username>
        // - .quit
        // - .color <r> <g> <b>
        // - <other text is send as message>

        let regex_expr = r"((?<cmd>.file|.image|.username) (?<name>.+)|(?<quit>.quit)|(?<color>.color (?<r>(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)) (?<g>(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)) (?<b>(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)))|(?<text>.+))";

        let Ok(re) = Regex::new(regex_expr) else {
            return Err(FromStrError::RegexCreate);
        };

        // this would be better to do by hand without regex. Then it would be possible to provide
        // better error messages to users
        let Some(caps) = re.captures(string) else {
            return Err(FromStrError::RegexParse);
        };

        match &caps.name("cmd") {
            Some(cmd) => match cmd.as_str() {
                ".file" => {
                    return Ok(CommandType::File(caps["name"].to_string()));
                }
                ".image" => {
                    return Ok(CommandType::Image(caps["name"].to_string()));
                }
                ".username" => {
                    return Ok(CommandType::Username(caps["name"].to_string()));
                }
                _ => {
                    return Err(FromStrError::Internal(
                        "This should never happen.".to_string(),
                    ));
                }
            },
            None => {}
        }

        match &caps.name("quit") {
            Some(_) => {
                return Ok(CommandType::Quit);
            }
            None => {}
        }

        match &caps.name("color") {
            Some(_) => {
                let Ok(r) = u8::from_str_radix(&caps["r"], 10) else {
                    return Err(FromStrError::StringToNumber);
                };
                let Ok(g) = u8::from_str_radix(&caps["g"], 10) else {
                    return Err(FromStrError::StringToNumber);
                };
                let Ok(b) = u8::from_str_radix(&caps["b"], 10) else {
                    return Err(FromStrError::StringToNumber);
                };
                return Ok(CommandType::Color((r, g, b)));
            }
            None => {}
        }

        // there should not be any other option
        return Ok(CommandType::Text(caps["text"].to_string()));
    }
}
