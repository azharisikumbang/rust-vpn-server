pub fn parse_message(input: &str) -> Result<(usize, &str), String> {
    if input.is_empty() {
        return Err("input cannot be empty".to_string());
    }

    let (tgt, msg) = match input.split_once(' ') {
        None => ("",""),
        Some((a, b)) => (a,b),
    };

    if msg.is_empty() {
        return Err("Error: missing message".to_string());
    }

    let tgt_id = match tgt.parse::<usize>() {
        Ok(val) => val,
        Err(_) => {
            return Err("Error: invalid client ID".to_string());
        }
    };

    Ok((tgt_id, msg))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_message_with_target_and_message() {
        let (target_id, message) = parse_message("2 valid message").unwrap();
        
        assert_eq!(target_id, 2);
        assert_eq!(message, "valid message");
    }

    #[test]
    fn test_parse_message_with_no_message() {
        match parse_message("2") {
            Err(err) => {      
                assert_eq!(err, "Error: missing message");
            },
            Ok(_) => {}
        };    
    }

    #[test]
    fn test_parse_message_with_invalid_target_id() {
        match parse_message("SALAH test") {
            Err(err) => {
                assert_eq!(err, "Error: invalid client ID");
            },
            Ok(_) => {}
        }
    }   
}