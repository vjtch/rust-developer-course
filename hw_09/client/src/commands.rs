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

#[derive(Debug, PartialEq)]
pub enum LogRegCommandType {
    Login(String, String),
    Register(String, String, String, u8, u8, u8),
}

impl FromStr for LogRegCommandType {
    type Err = FromStrError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        // parse input from user by using regex. There are two options:
        // - .login <username> <password>
        // - .register <username> <password> <password> <r> <g> <b>

        let regex_expr = r"(((?<login>\.login) (?<log_username>.+) (?<log_password>.+))|((?<register>\.register) (?<reg_username>.+) (?<reg_password>.+) (?<reg_repassword>.+) (?<r>(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)) (?<g>(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)) (?<b>(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?))))";

        let Ok(re) = Regex::new(regex_expr) else {
            return Err(FromStrError::RegexCreate);
        };

        // this would be better to do by hand without regex. Then it would be possible to provide
        // better error messages to users
        let Some(caps) = re.captures(string) else {
            return Err(FromStrError::RegexParse);
        };

        if caps.name("login").is_some() {
            return Ok(LogRegCommandType::Login(
                caps["log_username"].to_string(),
                caps["log_password"].to_string(),
            ));
        }

        if caps.name("register").is_some() {
            let Ok(r) = u8::from_str_radix(&caps["r"], 10) else {
                return Err(FromStrError::StringToNumber);
            };
            let Ok(g) = u8::from_str_radix(&caps["g"], 10) else {
                return Err(FromStrError::StringToNumber);
            };
            let Ok(b) = u8::from_str_radix(&caps["b"], 10) else {
                return Err(FromStrError::StringToNumber);
            };

            return Ok(LogRegCommandType::Register(
                caps["reg_username"].to_string(),
                caps["reg_password"].to_string(),
                caps["reg_repassword"].to_string(),
                r,
                g,
                b,
            ));
        }

        // Should never return this error. If there is invalid command, then it is handled by
        // `RegexParse` error. This error could occur if some of valid commands are not
        // handled in code above below `RegexParse` check.
        Err(FromStrError::Internal("Invalid command.".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_login_command_type_from_string_returns_ok() {
        let input = ".login username password";
        let expected = LogRegCommandType::Login("username".to_string(), "password".to_string());

        let actual = LogRegCommandType::from_str(input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn create_login_command_type_from_string_returns_err() {
        let input = ".login username";
        let expected = FromStrError::RegexParse;

        let actual = LogRegCommandType::from_str(input).unwrap_err();

        assert_eq!(actual, expected);
    }

    #[test]
    fn create_register_command_type_from_string_returns_ok() {
        let input = ".register username password password 0 0 255";
        let expected = LogRegCommandType::Register(
            "username".to_string(),
            "password".to_string(),
            "password".to_string(),
            0,
            0,
            255,
        );

        let actual = LogRegCommandType::from_str(input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn create_register_command_type_from_string_returns_err() -> Result<(), String> {
        let input = ".register username password password 0 0 1337";
        let expected = FromStrError::Internal("Invalid command.".to_string());

        match LogRegCommandType::from_str(input) {
            Ok(..) => Err("Should return invalid command error.".to_string()),
            Err(actual) => {
                assert_eq!(actual, expected);
                Ok(())
            }
        }
    }

    #[test]
    fn create_unknown_command_type_from_string_returns_err() {
        let input = ".help";
        let expected = FromStrError::RegexParse;

        let actual = LogRegCommandType::from_str(input).unwrap_err();

        assert_eq!(actual, expected);
    }
}
