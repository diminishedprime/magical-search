use std::{collections::HashSet, fmt};

use nom::{branch::alt, multi::many1, IResult, Parser};
use nom_supreme::{error::ErrorTree, tag::complete::tag_no_case, ParserExt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorOperand {
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

impl Ord for ColorOperand {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order().cmp(&other.order())
    }
}
impl PartialOrd for ColorOperand {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.order().partial_cmp(&other.order())
    }
}

impl ColorOperand {
    pub fn as_set(&self) -> HashSet<String> {
        HashSet::from_iter(self.as_vec())
    }
    pub fn as_vec(&self) -> Vec<String> {
        self.to_string().chars().map(|c| c.to_string()).collect()
    }
    fn order(&self) -> usize {
        match self {
            ColorOperand::White => 0,
            ColorOperand::Blue => 1,
            ColorOperand::Black => 2,
            ColorOperand::Red => 3,
            ColorOperand::Green => 4,
            // Technically we shouldn't be sorting Color::Esper, but this will
            // be a good enough workaround.
            _ => 5,
        }
    }
    fn collapse(mut colors: Vec<ColorOperand>) -> ColorOperand {
        colors.sort();
        colors.dedup();
        match colors.as_slice() {
            [ColorOperand::White, ColorOperand::Blue, ColorOperand::Black, ColorOperand::Red, ColorOperand::Green] => {
                ColorOperand::WUBRG
            }
            [ColorOperand::White, ColorOperand::Black, ColorOperand::Red, ColorOperand::Green] => {
                ColorOperand::Aggression
            }
            [ColorOperand::White, ColorOperand::Blue, ColorOperand::Red, ColorOperand::Green] => {
                ColorOperand::Altruism
            }
            [ColorOperand::White, ColorOperand::Blue, ColorOperand::Black, ColorOperand::Green] => {
                ColorOperand::Growth
            }
            [ColorOperand::White, ColorOperand::Blue, ColorOperand::Black, ColorOperand::Red] => {
                ColorOperand::Artifice
            }
            [ColorOperand::White, ColorOperand::Blue, ColorOperand::Black] => ColorOperand::Esper,
            [ColorOperand::White, ColorOperand::Blue, ColorOperand::Red] => ColorOperand::Jeskai,
            [ColorOperand::White, ColorOperand::Blue, ColorOperand::Green] => ColorOperand::Bant,
            [ColorOperand::White, ColorOperand::Black, ColorOperand::Red] => ColorOperand::Mardu,
            [ColorOperand::White, ColorOperand::Black, ColorOperand::Green] => ColorOperand::Abzan,
            [ColorOperand::White, ColorOperand::Red, ColorOperand::Green] => ColorOperand::Naya,
            [ColorOperand::White, ColorOperand::Blue] => ColorOperand::Azorius,
            [ColorOperand::White, ColorOperand::Black] => ColorOperand::Orzhov,
            [ColorOperand::White, ColorOperand::Red] => ColorOperand::Boros,
            [ColorOperand::White, ColorOperand::Green] => ColorOperand::Selesnya,
            [ColorOperand::Blue, ColorOperand::Black, ColorOperand::Red, ColorOperand::Green] => {
                ColorOperand::Grixis
            }
            [ColorOperand::Blue, ColorOperand::Black, ColorOperand::Red] => ColorOperand::Dimir,
            [ColorOperand::Blue, ColorOperand::Black, ColorOperand::Green] => ColorOperand::Golgari,
            [ColorOperand::Blue, ColorOperand::Red, ColorOperand::Green] => ColorOperand::Izzet,
            [ColorOperand::Blue, ColorOperand::Black] => ColorOperand::Dimir,
            [ColorOperand::Blue, ColorOperand::Red] => ColorOperand::Izzet,
            [ColorOperand::Blue, ColorOperand::Green] => ColorOperand::Simic,
            [ColorOperand::Black, ColorOperand::Red, ColorOperand::Green] => ColorOperand::Jund,
            [ColorOperand::Black, ColorOperand::Red] => ColorOperand::Rakdos,
            [ColorOperand::Black, ColorOperand::Green] => ColorOperand::Golgari,
            [ColorOperand::Red, ColorOperand::Green] => ColorOperand::Gruul,
            [ColorOperand::White] => ColorOperand::White,
            [ColorOperand::Blue] => ColorOperand::Blue,
            [ColorOperand::Black] => ColorOperand::Black,
            [ColorOperand::Red] => ColorOperand::Red,
            [ColorOperand::Green] => ColorOperand::Green,
            _ => panic!("Invalid color combination: {:?}", colors),
        }
    }
}

impl fmt::Display for ColorOperand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            ColorOperand::White => "W",
            ColorOperand::Blue => "U",
            ColorOperand::Black => "B",
            ColorOperand::Red => "R",
            ColorOperand::Green => "G",
            ColorOperand::Azorius => "WU",
            ColorOperand::Boros => "WR",
            ColorOperand::Dimir => "UB",
            ColorOperand::Golgari => "BG",
            ColorOperand::Gruul => "RG",
            ColorOperand::Izzet => "UR",
            ColorOperand::Orzhov => "WB",
            ColorOperand::Rakdos => "BR",
            ColorOperand::Selesnya => "WG",
            ColorOperand::Simic => "UG",
            ColorOperand::Colorless => "C",
            ColorOperand::Multicolor => "M",
            ColorOperand::Abzan => "WBG",
            ColorOperand::Jeskai => "WUR",
            ColorOperand::Sultai => "UBG",
            ColorOperand::Mardu => "WRB",
            ColorOperand::Temur => "URG",
            ColorOperand::Bant => "WUG",
            ColorOperand::Esper => "WUB",
            ColorOperand::Grixis => "UBR",
            ColorOperand::Jund => "BRG",
            ColorOperand::Naya => "WRG",
            ColorOperand::WUBRG => "WUBRG",
            ColorOperand::Aggression => "WBRG",
            ColorOperand::Altruism => "WURG",
            ColorOperand::Growth => "WUBG",
            ColorOperand::Artifice => "WUBR",
        };
        write!(f, "{}", symbol)
    }
}

fn color_combinations(input: &str) -> IResult<&str, ColorOperand, ErrorTree<&str>> {
    many1(basic_color).map(ColorOperand::collapse).parse(input)
}

fn basic_color(input: &str) -> IResult<&str, ColorOperand, ErrorTree<&str>> {
    alt((
        tag_no_case("w").value(ColorOperand::White),
        tag_no_case("u").value(ColorOperand::Blue),
        tag_no_case("b").value(ColorOperand::Black),
        tag_no_case("r").value(ColorOperand::Red),
        tag_no_case("g").value(ColorOperand::Green),
    ))
    .parse(input)
}

fn color_1(input: &str) -> IResult<&str, ColorOperand, ErrorTree<&str>> {
    alt((
        tag_no_case("abzan").value(ColorOperand::Abzan),
        tag_no_case("azorius").value(ColorOperand::Azorius),
        tag_no_case("bant").value(ColorOperand::Bant),
        tag_no_case("black").value(ColorOperand::Black),
        tag_no_case("blue").value(ColorOperand::Blue),
        tag_no_case("boros").value(ColorOperand::Boros),
        tag_no_case("colorless").value(ColorOperand::Colorless),
        tag_no_case("dimir").value(ColorOperand::Dimir),
        tag_no_case("esper").value(ColorOperand::Esper),
    ))
    .parse(input)
}
fn color_2(input: &str) -> IResult<&str, ColorOperand, ErrorTree<&str>> {
    alt((
        tag_no_case("golgari").value(ColorOperand::Golgari),
        tag_no_case("green").value(ColorOperand::Green),
        tag_no_case("grixis").value(ColorOperand::Grixis),
        tag_no_case("gruul").value(ColorOperand::Gruul),
        tag_no_case("izzet").value(ColorOperand::Izzet),
        tag_no_case("jeskai").value(ColorOperand::Jeskai),
        tag_no_case("jund").value(ColorOperand::Jund),
        tag_no_case("mardu").value(ColorOperand::Mardu),
        tag_no_case("multicolor").value(ColorOperand::Multicolor),
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
pub fn color(input: &str) -> IResult<&str, ColorOperand, ErrorTree<&str>> {
    // I can "only" parse 21 at a time, so doing these in chunks of 10.
    alt((
        color_1,
        color_2,
        tag_no_case("naya").value(ColorOperand::Naya),
        tag_no_case("orzhov").value(ColorOperand::Orzhov),
        tag_no_case("red").value(ColorOperand::Red),
        tag_no_case("rakdos").value(ColorOperand::Rakdos),
        tag_no_case("selesnya").value(ColorOperand::Selesnya),
        tag_no_case("simic").value(ColorOperand::Simic),
        tag_no_case("sultai").value(ColorOperand::Sultai),
        tag_no_case("temur").value(ColorOperand::Temur),
        tag_no_case("white").value(ColorOperand::White),
        tag_no_case("c").value(ColorOperand::Colorless),
        tag_no_case("m").value(ColorOperand::Multicolor),
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
        assert_eq!(color, ColorOperand::Abzan);
    }

    #[test]
    fn test_parse_color_azorius() {
        let (_, color) = color("azorius").unwrap();
        assert_eq!(color, ColorOperand::Azorius);
    }

    #[test]
    fn test_parse_color_b() {
        let (_, color) = color("b").unwrap();
        assert_eq!(color, ColorOperand::Black);
    }

    #[test]
    fn test_parse_color_bant() {
        let (_, color) = color("bant").unwrap();
        assert_eq!(color, ColorOperand::Bant);
    }

    #[test]
    fn test_parse_color_black() {
        let (_, color) = color("black").unwrap();
        assert_eq!(color, ColorOperand::Black);
    }

    #[test]
    fn test_parse_color_blue() {
        let (_, color) = color("blue").unwrap();
        assert_eq!(color, ColorOperand::Blue);
    }

    #[test]
    fn test_parse_color_boros() {
        let (_, color) = color("boros").unwrap();
        assert_eq!(color, ColorOperand::Boros);
    }

    #[test]
    fn test_parse_color_c() {
        let (_, color) = color("c").unwrap();
        assert_eq!(color, ColorOperand::Colorless);
    }

    #[test]
    fn test_parse_color_colorless() {
        let (_, color) = color("colorless").unwrap();
        assert_eq!(color, ColorOperand::Colorless);
    }

    #[test]
    fn test_parse_color_dimir() {
        let (_, color) = color("dimir").unwrap();
        assert_eq!(color, ColorOperand::Dimir);
    }

    #[test]
    fn test_parse_color_esper() {
        let (_, color) = color("esper").unwrap();
        assert_eq!(color, ColorOperand::Esper);
    }

    #[test]
    fn test_parse_color_g() {
        let (_, color) = color("g").unwrap();
        assert_eq!(color, ColorOperand::Green);
    }

    #[test]
    fn test_parse_color_golgari() {
        let (_, color) = color("golgari").unwrap();
        assert_eq!(color, ColorOperand::Golgari);
    }

    #[test]
    fn test_parse_color_green() {
        let (_, color) = color("green").unwrap();
        assert_eq!(color, ColorOperand::Green);
    }

    #[test]
    fn test_parse_color_grixis() {
        let (_, color) = color("grixis").unwrap();
        assert_eq!(color, ColorOperand::Grixis);
    }

    #[test]
    fn test_parse_color_gruul() {
        let (_, color) = color("gruul").unwrap();
        assert_eq!(color, ColorOperand::Gruul);
    }

    #[test]
    fn test_parse_color_izzet() {
        let (_, color) = color("izzet").unwrap();
        assert_eq!(color, ColorOperand::Izzet);
    }

    #[test]
    fn test_parse_color_jeskai() {
        let (_, color) = color("jeskai").unwrap();
        assert_eq!(color, ColorOperand::Jeskai);
    }

    #[test]
    fn test_parse_color_jund() {
        let (_, color) = color("jund").unwrap();
        assert_eq!(color, ColorOperand::Jund);
    }

    #[test]
    fn test_parse_color_m() {
        let (_, color) = color("m").unwrap();
        assert_eq!(color, ColorOperand::Multicolor);
    }

    #[test]
    fn test_parse_color_mardu() {
        let (_, color) = color("mardu").unwrap();
        assert_eq!(color, ColorOperand::Mardu);
    }

    #[test]
    fn test_parse_color_multicolor() {
        let (_, color) = color("multicolor").unwrap();
        assert_eq!(color, ColorOperand::Multicolor);
    }

    #[test]
    fn test_parse_color_naya() {
        let (_, color) = color("naya").unwrap();
        assert_eq!(color, ColorOperand::Naya);
    }

    #[test]
    fn test_parse_color_orzhov() {
        let (_, color) = color("orzhov").unwrap();
        assert_eq!(color, ColorOperand::Orzhov);
    }

    #[test]
    fn test_parse_color_r() {
        let (_, color) = color("r").unwrap();
        assert_eq!(color, ColorOperand::Red);
    }

    #[test]
    fn test_parse_color_rakdos() {
        let (_, color) = color("rakdos").unwrap();
        assert_eq!(color, ColorOperand::Rakdos);
    }

    #[test]
    fn test_parse_color_red() {
        let (_, color) = color("red").unwrap();
        assert_eq!(color, ColorOperand::Red);
    }

    #[test]
    fn test_parse_color_selesnya() {
        let (_, color) = color("selesnya").unwrap();
        assert_eq!(color, ColorOperand::Selesnya);
    }

    #[test]
    fn test_parse_color_simic() {
        let (_, color) = color("simic").unwrap();
        assert_eq!(color, ColorOperand::Simic);
    }

    #[test]
    fn test_parse_color_sultai() {
        let (_, color) = color("sultai").unwrap();
        assert_eq!(color, ColorOperand::Sultai);
    }

    #[test]
    fn test_parse_color_temur() {
        let (_, color) = color("temur").unwrap();
        assert_eq!(color, ColorOperand::Temur);
    }

    #[test]
    fn test_parse_color_u() {
        let (_, color) = color("u").unwrap();
        assert_eq!(color, ColorOperand::Blue);
    }

    #[test]
    fn test_parse_color_w() {
        let (_, color) = color("w").unwrap();
        assert_eq!(color, ColorOperand::White);
    }

    #[test]
    fn test_parse_color_white() {
        let (_, color) = color("white").unwrap();
        assert_eq!(color, ColorOperand::White);
    }

    #[test]
    fn test_parse_an_embarassment_of_white() {
        let (_, color) = color("WWWWWWWWWW").unwrap();
        assert_eq!(color, ColorOperand::White);
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
            ColorOperand::Green,
            ColorOperand::Red,
            ColorOperand::Black,
            ColorOperand::Blue,
            ColorOperand::White,
        ];
        colors.sort();
        assert_eq!(
            colors,
            vec![
                ColorOperand::White,
                ColorOperand::Blue,
                ColorOperand::Black,
                ColorOperand::Red,
                ColorOperand::Green
            ]
        );
    }
}
