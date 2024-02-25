use std::fmt;

use nom::{branch::alt, multi::many1, IResult, Parser};
use nom_supreme::tag::complete::tag_no_case;

#[derive(Debug, PartialEq, Eq)]
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
    fn collapse(colors: Vec<Color>) -> Color {
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

fn parse_color_combinations(input: &str) -> IResult<&str, Color> {
    let (rest, mut colors) = many1(parse_basic_color)(input)?;
    colors.sort();
    colors.dedup();
    let color = Color::collapse(colors);
    Ok((rest, color))
}

fn parse_basic_color(input: &str) -> IResult<&str, Color> {
    alt((
        tag_no_case("w").map(|_| Color::White),
        tag_no_case("u").map(|_| Color::Blue),
        tag_no_case("b").map(|_| Color::Black),
        tag_no_case("r").map(|_| Color::Red),
        tag_no_case("g").map(|_| Color::Green),
    ))
    .parse(input)
}

fn parse_color_1(input: &str) -> IResult<&str, Color> {
    alt((
        tag_no_case("abzan").map(|_| Color::Abzan),
        tag_no_case("azorius").map(|_| Color::Azorius),
        tag_no_case("bant").map(|_| Color::Bant),
        tag_no_case("black").map(|_| Color::Black),
        tag_no_case("blue").map(|_| Color::Blue),
        tag_no_case("boros").map(|_| Color::Boros),
        tag_no_case("colorless").map(|_| Color::Colorless),
        tag_no_case("dimir").map(|_| Color::Dimir),
        tag_no_case("esper").map(|_| Color::Esper),
    ))(input)
}
fn parse_color_2(input: &str) -> IResult<&str, Color> {
    alt((
        tag_no_case("golgari").map(|_| Color::Golgari),
        tag_no_case("green").map(|_| Color::Green),
        tag_no_case("grixis").map(|_| Color::Grixis),
        tag_no_case("gruul").map(|_| Color::Gruul),
        tag_no_case("izzet").map(|_| Color::Izzet),
        tag_no_case("jeskai").map(|_| Color::Jeskai),
        tag_no_case("jund").map(|_| Color::Jund),
        tag_no_case("mardu").map(|_| Color::Mardu),
        tag_no_case("multicolor").map(|_| Color::Multicolor),
    ))(input)
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
pub fn parse_color(input: &str) -> IResult<&str, Color> {
    // I can "only" parse 21 at a time, so doing these in chunks of 10.
    alt((
        parse_color_1,
        parse_color_2,
        tag_no_case("naya").map(|_| Color::Naya),
        tag_no_case("orzhov").map(|_| Color::Orzhov),
        tag_no_case("red").map(|_| Color::Red),
        tag_no_case("rakdos").map(|_| Color::Rakdos),
        tag_no_case("selesnya").map(|_| Color::Selesnya),
        tag_no_case("simic").map(|_| Color::Simic),
        tag_no_case("sultai").map(|_| Color::Sultai),
        tag_no_case("temur").map(|_| Color::Temur),
        tag_no_case("white").map(|_| Color::White),
        tag_no_case("c").map(|_| Color::Colorless),
        tag_no_case("m").map(|_| Color::Multicolor),
        parse_color_combinations,
    ))(input)
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use nom::combinator::complete;

    use super::*;

    #[test]
    fn test_parse_color_abzan() {
        assert_eq!(parse_color("abzan"), Ok(("", Color::Abzan)));
    }

    #[test]
    fn test_parse_color_azorius() {
        assert_eq!(parse_color("azorius"), Ok(("", Color::Azorius)));
    }

    #[test]
    fn test_parse_color_b() {
        assert_eq!(parse_color("b"), Ok(("", Color::Black)));
    }

    #[test]
    fn test_parse_color_bant() {
        assert_eq!(parse_color("bant"), Ok(("", Color::Bant)));
    }

    #[test]
    fn test_parse_color_black() {
        assert_eq!(parse_color("black"), Ok(("", Color::Black)));
    }

    #[test]
    fn test_parse_color_blue() {
        assert_eq!(parse_color("blue"), Ok(("", Color::Blue)));
    }

    #[test]
    fn test_parse_color_boros() {
        assert_eq!(parse_color("boros"), Ok(("", Color::Boros)));
    }

    #[test]
    fn test_parse_color_c() {
        assert_eq!(parse_color("c"), Ok(("", Color::Colorless)));
    }

    #[test]
    fn test_parse_color_colorless() {
        assert_eq!(parse_color("colorless"), Ok(("", Color::Colorless)));
    }

    #[test]
    fn test_parse_color_dimir() {
        assert_eq!(parse_color("dimir"), Ok(("", Color::Dimir)));
    }

    #[test]
    fn test_parse_color_esper() {
        assert_eq!(parse_color("esper"), Ok(("", Color::Esper)));
    }

    #[test]
    fn test_parse_color_g() {
        assert_eq!(parse_color("g"), Ok(("", Color::Green)));
    }

    #[test]
    fn test_parse_color_golgari() {
        assert_eq!(parse_color("golgari"), Ok(("", Color::Golgari)));
    }

    #[test]
    fn test_parse_color_green() {
        assert_eq!(parse_color("green"), Ok(("", Color::Green)));
    }

    #[test]
    fn test_parse_color_grixis() {
        assert_eq!(parse_color("grixis"), Ok(("", Color::Grixis)));
    }

    #[test]
    fn test_parse_color_gruul() {
        assert_eq!(parse_color("gruul"), Ok(("", Color::Gruul)));
    }

    #[test]
    fn test_parse_color_izzet() {
        assert_eq!(parse_color("izzet"), Ok(("", Color::Izzet)));
    }

    #[test]
    fn test_parse_color_jeskai() {
        assert_eq!(parse_color("jeskai"), Ok(("", Color::Jeskai)));
    }

    #[test]
    fn test_parse_color_jund() {
        assert_eq!(parse_color("jund"), Ok(("", Color::Jund)));
    }

    #[test]
    fn test_parse_color_m() {
        assert_eq!(parse_color("m"), Ok(("", Color::Multicolor)));
    }

    #[test]
    fn test_parse_color_mardu() {
        assert_eq!(parse_color("mardu"), Ok(("", Color::Mardu)));
    }

    #[test]
    fn test_parse_color_multicolor() {
        assert_eq!(parse_color("multicolor"), Ok(("", Color::Multicolor)));
    }

    #[test]
    fn test_parse_color_naya() {
        assert_eq!(parse_color("naya"), Ok(("", Color::Naya)));
    }

    #[test]
    fn test_parse_color_orzhov() {
        assert_eq!(parse_color("orzhov"), Ok(("", Color::Orzhov)));
    }

    #[test]
    fn test_parse_color_r() {
        assert_eq!(parse_color("r"), Ok(("", Color::Red)));
    }

    #[test]
    fn test_parse_color_rakdos() {
        assert_eq!(parse_color("rakdos"), Ok(("", Color::Rakdos)));
    }

    #[test]
    fn test_parse_color_red() {
        assert_eq!(parse_color("red"), Ok(("", Color::Red)));
    }

    #[test]
    fn test_parse_color_selesnya() {
        assert_eq!(parse_color("selesnya"), Ok(("", Color::Selesnya)));
    }

    #[test]
    fn test_parse_color_simic() {
        assert_eq!(parse_color("simic"), Ok(("", Color::Simic)));
    }

    #[test]
    fn test_parse_color_sultai() {
        assert_eq!(parse_color("sultai"), Ok(("", Color::Sultai)));
    }

    #[test]
    fn test_parse_color_temur() {
        assert_eq!(parse_color("temur"), Ok(("", Color::Temur)));
    }

    #[test]
    fn test_parse_color_u() {
        assert_eq!(parse_color("u"), Ok(("", Color::Blue)));
    }

    #[test]
    fn test_parse_color_w() {
        assert_eq!(parse_color("w"), Ok(("", Color::White)));
    }

    #[test]
    fn test_parse_color_white() {
        assert_eq!(parse_color("white"), Ok(("", Color::White)));
    }

    #[test]
    fn test_parse_an_embarassment_of_white() {
        assert_eq!(parse_color("WWWWWWWWWW"), Ok(("", Color::White)));
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
            let result = complete(parse_color)(&input);
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
