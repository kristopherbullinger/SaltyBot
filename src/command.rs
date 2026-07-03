use std::convert::TryFrom;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command<'a> {
    Friday,
    Glossary(&'a str),
}

impl<'a> TryFrom<&'a str> for Command<'a> {
    type Error = ();
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        //attempt to parse Friday
        let lowered = s.to_ascii_lowercase();
        if lowered.starts_with("is it friday") {
            return Ok(Command::Friday);
        }
        //attempt to parse Glossary
        if lowered.starts_with("-glossary") {
            let (_, rest) = s.split_at("-glossary".len());
            return Ok(Command::Glossary(rest.trim()));
        }
        Err(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn command_from_str_properly() {
        let cases = &[
            ("is it friday", Ok(Command::Friday)),
            ("IS IT FRIDAY", Ok(Command::Friday)),
            ("Is It Friday??", Ok(Command::Friday)),
            ("123  we qerqe", Err(())),
            ("-glossary", Ok(Command::Glossary(""))),
            ("Nothing", Err(())),
        ];
        for case in cases.iter() {
            assert_eq!(Command::try_from(case.0), case.1);
        }
    }
}
