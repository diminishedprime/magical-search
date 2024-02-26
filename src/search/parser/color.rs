use std::{collections::HashSet, fmt};

use nom::{branch::alt, multi::many1, IResult, Parser};
use nom_supreme::{error::ErrorTree, tag::complete::tag_no_case, ParserExt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Color {
    Red,
    Blue,
    Black,
    Green,
    White,
    Azorius,
    Boros,
    Dimir,
    Golgari,
    Gruul,
    Izzet,
    Orzhov,
    Rakdos,
    Selesnya,
    Simic,
    Colorless,
    Multicolor,
    Abzan,
    Jeskai,
    Sultai,
    Mardu,
    Temur,
    Bant,
    Esper,
    Grixis,
    Jund,
    Naya,
    Aggression, // (black/red/green/white)
    Altruism,   // (red/green/white/blue)
    Growth,     // (green/white/blue/black)
    Artifice,   // (white/blue/black/red)
    WUBRG,
}

impl Ord for Color {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order().cmp(&other.order())
    }
}
impl PartialOrd for Color {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.order().partial_cmp(&other.order())
    }
}

impl Color {
    pub fn as_set(&self) -> HashSet<String> {
        HashSet::from_iter(self.as_vec())
    }
    pub fn as_vec(&self) -> Vec<String> {
        self.to_string().chars().map(|c| c.to_string()).collect()
    }
    fn order(&self) -> usize {
        match self {
            Color::White => 0,
            Color::Blue => 1,
            Color::Black => 2,
            Color::Red => 3,
            Color::Green => 4,
            // Technically we shouldn't be sorting Color::Esper, but this will
            // be a good enough workaround.
            _ => 5,
        }
    }
    fn collapse(mut colors: Vec<Color>) -> Color {
        colors.sort();
        colors.dedup();
        match colors.as_slice() {
            [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] => Color::WUBRG,
            [Color::White, Color::Black, Color::Red, Color::Green] => Color::Aggression,
            [Color::White, Color::Blue, Color::Red, Color::Green] => Color::Altruism,
            [Color::White, Color::Blue, Color::Black, Color::Green] => Color::Growth,
            [Color::White, Color::Blue, Color::Black, Color::Red] => Color::Artifice,
            [Color::White, Color::Blue, Color::Black] => Color::Esper,
            [Color::White, Color::Blue, Color::Red] => Color::Jeskai,
            [Color::White, Color::Blue, Color::Green] => Color::Bant,
            [Color::White, Color::Black, Color::Red] => Color::Mardu,
            [Color::White, Color::Black, Color::Green] => Color::Abzan,
            [Color::White, Color::Red, Color::Green] => Color::Naya,
            [Color::White, Color::Blue] => Color::Azorius,
            [Color::White, Color::Black] => Color::Orzhov,
            [Color::White, Color::Red] => Color::Boros,
            [Color::White, Color::Green] => Color::Selesnya,
            [Color::Blue, Color::Black, Color::Red, Color::Green] => Color::Grixis,
            [Color::Blue, Color::Black, Color::Red] => Color::Dimir,
            [Color::Blue, Color::Black, Color::Green] => Color::Golgari,
            [Color::Blue, Color::Red, Color::Green] => Color::Izzet,
            [Color::Blue, Color::Black] => Color::Dimir,
            [Color::Blue, Color::Red] => Color::Izzet,
            [Color::Blue, Color::Green] => Color::Simic,
            [Color::Black, Color::Red, Color::Green] => Color::Jund,
            [Color::Black, Color::Red] => Color::Rakdos,
            [Color::Black, Color::Green] => Color::Golgari,
            [Color::Red, Color::Green] => Color::Gruul,
            [Color::White] => Color::White,
            [Color::Blue] => Color::Blue,
            [Color::Black] => Color::Black,
            [Color::Red] => Color::Red,
            [Color::Green] => Color::Green,
            _ => panic!("Invalid color combination: {:?}", colors),
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            Color::White => "W",
            Color::Blue => "U",
            Color::Black => "B",
            Color::Red => "R",
            Color::Green => "G",
            Color::Azorius => "WU",
            Color::Boros => "WR",
            Color::Dimir => "UB",
            Color::Golgari => "BG",
            Color::Gruul => "RG",
            Color::Izzet => "UR",
            Color::Orzhov => "WB",
            Color::Rakdos => "BR",
            Color::Selesnya => "WG",
            Color::Simic => "UG",
            Color::Colorless => "C",
            Color::Multicolor => "M",
            Color::Abzan => "WBG",
            Color::Jeskai => "WUR",
            Color::Sultai => "UBG",
            Color::Mardu => "WRB",
            Color::Temur => "URG",
            Color::Bant => "WUG",
            Color::Esper => "WUB",
            Color::Grixis => "UBR",
            Color::Jund => "BRG",
            Color::Naya => "WRG",
            Color::WUBRG => "WUBRG",
            Color::Aggression => "WBRG",
            Color::Altruism => "WURG",
            Color::Growth => "WUBG",
            Color::Artifice => "WUBR",
        };
        write!(f, "{}", symbol)
    }
}

fn color_combinations(input: &str) -> IResult<&str, Color, ErrorTree<&str>> {
    many1(basic_color).map(Color::collapse).parse(input)
}

fn basic_color(input: &str) -> IResult<&str, Color, ErrorTree<&str>> {
    alt((
        tag_no_case("w").value(Color::White),
        tag_no_case("u").value(Color::Blue),
        tag_no_case("b").value(Color::Black),
        tag_no_case("r").value(Color::Red),
        tag_no_case("g").value(Color::Green),
    ))
    .parse(input)
}

fn color_1(input: &str) -> IResult<&str, Color, ErrorTree<&str>> {
    alt((
        tag_no_case("abzan").value(Color::Abzan),
        tag_no_case("azorius").value(Color::Azorius),
        tag_no_case("bant").value(Color::Bant),
        tag_no_case("black").value(Color::Black),
        tag_no_case("blue").value(Color::Blue),
        tag_no_case("boros").value(Color::Boros),
        tag_no_case("colorless").value(Color::Colorless),
        tag_no_case("dimir").value(Color::Dimir),
        tag_no_case("esper").value(Color::Esper),
    ))
    .parse(input)
}
fn color_2(input: &str) -> IResult<&str, Color, ErrorTree<&str>> {
    alt((
        tag_no_case("golgari").value(Color::Golgari),
        tag_no_case("green").value(Color::Green),
        tag_no_case("grixis").value(Color::Grixis),
        tag_no_case("gruul").value(Color::Gruul),
        tag_no_case("izzet").value(Color::Izzet),
        tag_no_case("jeskai").value(Color::Jeskai),
        tag_no_case("jund").value(Color::Jund),
        tag_no_case("mardu").value(Color::Mardu),
        tag_no_case("multicolor").value(Color::Multicolor),
    ))
    .parse(input)
}

// Both sets of keywords accepts full color names like blue or the abbreviated
// color letters w, u, r, b and g.
//
// You can use many nicknames for color sets: all guild names (e.g. azorius),
// all shard names (e.g. bant), all college names (e.g., quandrix), all wedge
// names (e.g. abzan), and the four-color nicknames chaos, aggression, altruism,
// growth, artifice are supported.
//
// Use c or colorless to match colorless cards, and m or multicolor to match
// multicolor cards.
pub fn color(input: &str) -> IResult<&str, Color, ErrorTree<&str>> {
    // I can "only" parse 21 at a time, so doing these in chunks of 10.
    alt((
        color_1,
        color_2,
        tag_no_case("naya").value(Color::Naya),
        tag_no_case("orzhov").value(Color::Orzhov),
        tag_no_case("red").value(Color::Red),
        tag_no_case("rakdos").value(Color::Rakdos),
        tag_no_case("selesnya").value(Color::Selesnya),
        tag_no_case("simic").value(Color::Simic),
        tag_no_case("sultai").value(Color::Sultai),
        tag_no_case("temur").value(Color::Temur),
        tag_no_case("white").value(Color::White),
        tag_no_case("c").value(Color::Colorless),
        tag_no_case("m").value(Color::Multicolor),
        color_combinations,
    ))
    .parse(input)
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use nom::combinator::complete;

    use super::*;

    #[test]
    fn test_parse_color_abzan() {
        let (_, color) = color("abzan").unwrap();
        assert_eq!(color, Color::Abzan);
    }

    #[test]
    fn test_parse_color_azorius() {
        let (_, color) = color("azorius").unwrap();
        assert_eq!(color, Color::Azorius);
    }

    #[test]
    fn test_parse_color_b() {
        let (_, color) = color("b").unwrap();
        assert_eq!(color, Color::Black);
    }

    #[test]
    fn test_parse_color_bant() {
        let (_, color) = color("bant").unwrap();
        assert_eq!(color, Color::Bant);
    }

    #[test]
    fn test_parse_color_black() {
        let (_, color) = color("black").unwrap();
        assert_eq!(color, Color::Black);
    }

    #[test]
    fn test_parse_color_blue() {
        let (_, color) = color("blue").unwrap();
        assert_eq!(color, Color::Blue);
    }

    #[test]
    fn test_parse_color_boros() {
        let (_, color) = color("boros").unwrap();
        assert_eq!(color, Color::Boros);
    }

    #[test]
    fn test_parse_color_c() {
        let (_, color) = color("c").unwrap();
        assert_eq!(color, Color::Colorless);
    }

    #[test]
    fn test_parse_color_colorless() {
        let (_, color) = color("colorless").unwrap();
        assert_eq!(color, Color::Colorless);
    }

    #[test]
    fn test_parse_color_dimir() {
        let (_, color) = color("dimir").unwrap();
        assert_eq!(color, Color::Dimir);
    }

    #[test]
    fn test_parse_color_esper() {
        let (_, color) = color("esper").unwrap();
        assert_eq!(color, Color::Esper);
    }

    #[test]
    fn test_parse_color_g() {
        let (_, color) = color("g").unwrap();
        assert_eq!(color, Color::Green);
    }

    #[test]
    fn test_parse_color_golgari() {
        let (_, color) = color("golgari").unwrap();
        assert_eq!(color, Color::Golgari);
    }

    #[test]
    fn test_parse_color_green() {
        let (_, color) = color("green").unwrap();
        assert_eq!(color, Color::Green);
    }

    #[test]
    fn test_parse_color_grixis() {
        let (_, color) = color("grixis").unwrap();
        assert_eq!(color, Color::Grixis);
    }

    #[test]
    fn test_parse_color_gruul() {
        let (_, color) = color("gruul").unwrap();
        assert_eq!(color, Color::Gruul);
    }

    #[test]
    fn test_parse_color_izzet() {
        let (_, color) = color("izzet").unwrap();
        assert_eq!(color, Color::Izzet);
    }

    #[test]
    fn test_parse_color_jeskai() {
        let (_, color) = color("jeskai").unwrap();
        assert_eq!(color, Color::Jeskai);
    }

    #[test]
    fn test_parse_color_jund() {
        let (_, color) = color("jund").unwrap();
        assert_eq!(color, Color::Jund);
    }

    #[test]
    fn test_parse_color_m() {
        let (_, color) = color("m").unwrap();
        assert_eq!(color, Color::Multicolor);
    }

    #[test]
    fn test_parse_color_mardu() {
        let (_, color) = color("mardu").unwrap();
        assert_eq!(color, Color::Mardu);
    }

    #[test]
    fn test_parse_color_multicolor() {
        let (_, color) = color("multicolor").unwrap();
        assert_eq!(color, Color::Multicolor);
    }

    #[test]
    fn test_parse_color_naya() {
        let (_, color) = color("naya").unwrap();
        assert_eq!(color, Color::Naya);
    }

    #[test]
    fn test_parse_color_orzhov() {
        let (_, color) = color("orzhov").unwrap();
        assert_eq!(color, Color::Orzhov);
    }

    #[test]
    fn test_parse_color_r() {
        let (_, color) = color("r").unwrap();
        assert_eq!(color, Color::Red);
    }

    #[test]
    fn test_parse_color_rakdos() {
        let (_, color) = color("rakdos").unwrap();
        assert_eq!(color, Color::Rakdos);
    }

    #[test]
    fn test_parse_color_red() {
        let (_, color) = color("red").unwrap();
        assert_eq!(color, Color::Red);
    }

    #[test]
    fn test_parse_color_selesnya() {
        let (_, color) = color("selesnya").unwrap();
        assert_eq!(color, Color::Selesnya);
    }

    #[test]
    fn test_parse_color_simic() {
        let (_, color) = color("simic").unwrap();
        assert_eq!(color, Color::Simic);
    }

    #[test]
    fn test_parse_color_sultai() {
        let (_, color) = color("sultai").unwrap();
        assert_eq!(color, Color::Sultai);
    }

    #[test]
    fn test_parse_color_temur() {
        let (_, color) = color("temur").unwrap();
        assert_eq!(color, Color::Temur);
    }

    #[test]
    fn test_parse_color_u() {
        let (_, color) = color("u").unwrap();
        assert_eq!(color, Color::Blue);
    }

    #[test]
    fn test_parse_color_w() {
        let (_, color) = color("w").unwrap();
        assert_eq!(color, Color::White);
    }

    #[test]
    fn test_parse_color_white() {
        let (_, color) = color("white").unwrap();
        assert_eq!(color, Color::White);
    }

    #[test]
    fn test_parse_an_embarassment_of_white() {
        let (_, color) = color("WWWWWWWWWW").unwrap();
        assert_eq!(color, Color::White);
    }

    fn generate_color_combinations(num_colors: usize) -> Vec<Vec<String>> {
        let colors = vec![
            "w".to_string(),
            "u".to_string(),
            "b".to_string(),
            "r".to_string(),
            "g".to_string(),
        ];
        let mut combinations = vec![];
        for combination in colors.into_iter().combinations(num_colors) {
            combinations.push(combination.into_iter().collect::<Vec<_>>());
        }
        combinations
    }

    fn test_color_combinations(num_colors: usize) {
        let combinations = generate_color_combinations(num_colors);
        for combination in &combinations {
            let input = combination.join("");
            let result = complete(color)(&input);
            assert!(
                result.is_ok() && result.unwrap().0.is_empty(),
                "Failed to parse color combination: {}",
                input
            );
        }
    }

    #[test]
    fn test_parse_all_color_combinations() {
        test_color_combinations(5);
    }

    #[test]
    fn test_parse_all_4_combinations() {
        test_color_combinations(4);
    }
    #[test]
    fn test_parse_all_3_combinations() {
        test_color_combinations(3);
    }
    #[test]
    fn test_parse_all_2_combinations() {
        test_color_combinations(2);
    }

    #[test]
    fn test_parse_all_1_combinations() {
        test_color_combinations(1);
    }

    #[test]
    fn test_sort_colors() {
        let mut colors = vec![
            Color::Green,
            Color::Red,
            Color::Black,
            Color::Blue,
            Color::White,
        ];
        colors.sort();
        assert_eq!(
            colors,
            vec![
                Color::White,
                Color::Blue,
                Color::Black,
                Color::Red,
                Color::Green
            ]
        );
    }
}
