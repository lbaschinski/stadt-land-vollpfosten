use rand::seq::SliceRandom;

/// Special 30 sided dice from the game.
/// Is missing X and Y, therefore A, E, O and S are doubled.
/// Has two wildcard (⚡) sides as well.
const DICE: [&str; 30] =
    [ "A"
    , "A" // no error, is twice on the dice
    , "B"
    , "C"
    , "D"
    , "E"
    , "E" // no error, is twice on the dice
    , "F"
    , "G"
    , "H"
    , "I"
    , "J"
    , "K"
    , "L"
    , "M"
    , "N"
    , "O"
    , "O" // no error, is twice on the dice
    , "P"
    , "Q"
    , "R"
    , "S"
    , "S" // no error, is twice on the dice
    , "T"
    , "U"
    , "V"
    , "W"
    , "Z"
    , "⚡" // choose your own letter
    , "⚡" // no error, is twice on the dice
    ];

pub fn roll_dice() -> &'static str {
    let mut rng = rand::thread_rng();
    DICE.choose(&mut rng).unwrap()
}
