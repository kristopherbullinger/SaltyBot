use std::convert::TryFrom;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command<'a> {
    Friday,
    ListSelfAssignRoles,
    AddSelfAssignRole(AddSelfAssignRole<'a>),
    RemoveSelfAssignRole(&'a str),
    Glossary(&'a str),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AddSelfAssignRole<'a> {
    pub emoji: &'a str,
    pub role_name: &'a str,
}

impl<'a> TryFrom<&'a str> for Command<'a> {
    type Error = ();
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        if let Some(rest) = s.strip_prefix("-rmsar ") {
            return Ok(Command::RemoveSelfAssignRole(rest.trim()));
        }
        if let Some(asar) = parse_asar(s) {
            return Ok(Command::AddSelfAssignRole(asar));
        }
        //attempt to parse Friday
        let lowered = s.to_ascii_lowercase();
        if lowered.starts_with("is it friday") {
            return Ok(Command::Friday);
        }
        if lowered == "-lsar" {
            return Ok(Command::ListSelfAssignRoles);
        }
        //attempt to parse Glossary
        if lowered.starts_with("-glossary") {
            let (_, rest) = s.split_at("-glossary".len());
            return Ok(Command::Glossary(rest.trim()));
        }
        Err(())
    }
}

fn parse_asar(inp: &str) -> Option<AddSelfAssignRole> {
    let inp = inp.strip_prefix("-asar ")?;
    let emoji = inp.split_whitespace().next_back()?.trim();
    let role_name = inp[0..inp.len() - emoji.len()].trim();
    Some(AddSelfAssignRole { emoji, role_name })
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
