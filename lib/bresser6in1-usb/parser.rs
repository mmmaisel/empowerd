use super::message::Message;

pub enum ParserResult {
    CollectingData,
    Success(String),
}

pub enum ParserError {
    IgnoredMessage(String),
    Error(String),
}

pub struct Parser {
    buffer: String,
    last_fragment: u8,
}

impl Parser {
    pub fn new() -> Parser {
        return Parser {
            buffer: String::with_capacity(162),
            last_fragment: 0,
        };
    }

    pub fn parse_message(
        &mut self,
        message: Message,
    ) -> Result<ParserResult, ParserError> {
        if message.msgtype != 0xFE {
            return Err(ParserError::IgnoredMessage(format!(
                "Received unknown message type {}.",
                message.msgtype
            )));
        }

        match message.fragment as char {
            '1' => {
                self.buffer.clear();
                self.append_msg(message)?;
                self.last_fragment = '1' as u8;
                return Ok(ParserResult::CollectingData);
            }
            '2' => {
                if self.last_fragment != '1' as u8 {
                    return Err(ParserError::IgnoredMessage(
                        "Wrong fragment order".to_string(),
                    ));
                }
                self.append_msg(message)?;
                self.last_fragment = '2' as u8;
                return Ok(ParserResult::CollectingData);
            }
            '3' => {
                if self.last_fragment != '2' as u8 {
                    return Err(ParserError::IgnoredMessage(
                        "Wrong fragment order".to_string(),
                    ));
                }
                self.append_msg(message)?;
                return Ok(ParserResult::Success(self.buffer.clone()));
            }
            _ => {
                return Err(ParserError::IgnoredMessage(format!(
                    "Received unknown fragment type {}.",
                    message.fragment
                )));
            }
        }
    }

    fn append_msg(&mut self, message: Message) -> Result<(), ParserError> {
        return match message.to_string() {
            Ok(x) => {
                self.buffer.push_str(&x);
                Ok(())
            }
            Err(e) => Err(ParserError::Error(e.to_string())),
        };
    }
}
