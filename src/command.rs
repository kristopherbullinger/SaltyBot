use std::str::FromStr;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Salt,
    Friday,
}

impl FromStr for Command {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_ascii_lowercase();
        if s.starts_with("is it friday") {
            return Ok(Command::Friday);
        } else if s == "-salt" {
            return Ok(Command::Salt);
        }
        Err(())
    }
}

pub static FRIDAY_GIFS: &[&str] = &[
    "https://tenor.com/view/tekken-king-slap-fight-gif-14352072",
    "https://tenor.com/view/faust-guilty-gear-ky-kiske-gif-19883342",
    "https://tenor.com/view/tekken-king-slam-perfect-gif-13368205",
];

pub static QUOTES: &[&str] = &[
    "Complaining is saying \"chain grabs are fucking stupid\" which they aren't. \
    They're just incredibly braindead",
    "labbing is shit",
    "blocking is cowardly",
    "Enjoy Your Life In Hell Motherfucken Cheater You Must Really Enjoy Kissing \
    The Devil's Ass All You Did Was Stand There Cowardly Blocking An Doing The Same \
    Attack Over An Over Again You Must Really Want To Go To Hell Just To Win Over A \
    Dumb Game Well Have Fun Being Torture In Hell",
    "You're good in a dead game that no one will remember. Try doing it in a game \
    well respected in the competitive FPS community and I guarantee you won't last a year.",
    "he threw me 3 times in a row.\nthat merits a rage quit.",
    "Fighting games are stupid, unbalanced, not at all fun, and should have died in the 90s",
    "minus is unsafe",
    "Can't you just play without using exploits that the developers clearly \
    didn't thought about when making the rules?",
    "you only know how to grab and use the same move ur not good bud",
    "as long as you promise not to taunt, play too patiently to get under my skin, not \
    to have bad manners and not pick ken, necro, gouki, makoto, chun, yun, yang, oro, \
    ibuki or elena then yes\nwe can play",
    "Hot Take: You Arent Good If You Use a Grappler\ngetting punched in the face 300 times and \
    then calling heads on two coin flips with your last 10 hp does not make you good bro",
    "whoever made the first fighting game with a character built around spamming \
    and then calling it \"zoning\" and turned it into some cool archetype deserves to \
    be drawn and quartered in public",
    "This gigas is literally everything wrong with tekken atm,combos are such a crutch \
    because they give too much reward, so durrr launcher go brrr is actually really \
    a legitimate strategy",
    "losing doesn't make you a loser, wining unfairly does",
    "And i say it again all high tier players are assholes",
    "But telling me to go into the lab\nIs not helping me learn\nthat is being an asshole and \
    not wanting to help me",
    "1: why do you get launched\n2: Because the grab misses\n1: why does it miss\n2: because \
    It's never close enough\n1: why aren't you close enough\n2: I dunno",
    "I AM SO DONE WITH THIS GARBAGE ASS GAME\nAND FUCK YOU TOO",
    "I hate playin the Ditto, because it just comes down to better decision making",
    "1: i told you to block on wakeup and you said it \"doesn't work\"\n1: I also told \
    you you have an invincible dp\n2: It doesn;t\n2: His dp is ass\n1: His dp is invincible\n\
    2: It's ass",
    "If you're mad because I 'plugged' or whatever, chances are that you were: scrumming, \
    spamming, cowarding, a Paul player or just too boring a player to deal with. I will \
    always stay if it's a decent match, win or lose, but resort to either of the previous \
    and I don't mind taking a hit to the rank, it means very little to me.",
];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn command_from_str_properly() {
        let cases = &[
            ("-salt", Ok(Command::Salt)),
            ("is it friday", Ok(Command::Friday)),
            ("IS IT FRIDAY", Ok(Command::Friday)),
            ("Is It Friday??", Ok(Command::Friday)),
            ("123  we qerqe", Err(())),
            ("Nothing", Err(())),
        ];
        for case in cases.iter().copied() {
            assert_eq!(case.0.parse::<Command>(), case.1);
        }
    }
}
