use clap::Parser;
use std::fmt;
use moon_phase::MoonPhase;
use std::time::SystemTime;
use chrono::{Datelike,Timelike,DateTime,offset::Utc,TimeZone};
use human_date_parser::from_human_time;

// Unicode variation selectors (VS): these are invisible characters that will make the previous
// emoji show in text- or color presentation.
//
// If no VS is present it's up to the system how to display the emojis.
const VS15: &str = "\u{fe0e}"; // text emoji
const VS16: &str = "\u{fe0f}"; // color emoji
enum EmojiVariation {
    Unspecified,
    Text,
    Colour,
}

const NORTH_EMOJI: [&str; 8] = [
    "🌑",
    "🌒",
    "🌓",
    "🌔",
    "🌕",
    "🌖",
    "🌗",
    "🌘",
];
const SOUTH_EMOJI: [&str; 8] = [
    "🌑",
    "🌘",
    "🌗",
    "🌖",
    "🌕",
    "🌔",
    "🌓",
    "🌒",
];
const NORTH_EMOJI_FACE: [&str; 8] = [
    "🌚",
    "🌚",
    "🌛",
    "🌛",
    "🌝",
    "🌝",
    "🌜",
    "🌜",
];
const SOUTH_EMOJI_FACE: [&str; 8] = [
    "🌚",
    "🌚",
    "🌜",
    "🌜",
    "🌝",
    "🌝",
    "🌛",
    "🌛",
];

const NORTH_NERD_EMOJI: [&str; 28] = [
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
];

const SOUTH_NERD_EMOJI: [&str; 28] = [
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
    " ",
];

#[derive(clap::ValueEnum, Clone, Debug)]
enum Mode {
    Name,
    Emoji,
    NerdEmoji,
    Numeric,
}
impl std::fmt::Display for Mode {
    // Display the name of the enum value in lowercase
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s : String = format!("{:?}", self);
        write!(f, "{}", s.to_lowercase())
    }
}

#[derive(Parser)]
#[command(author,
          version,
          about="Show the moon phase as an emoji, number, or string.",
          max_term_width=80,
          long_about = None )]
struct Cli {
    /// Format in which to display the moon phase or moon sign.
    #[arg(short, long, default_value_t=Mode::Name,group="themode")]
    mode: Mode,

    // unnecessary, but I keep calling this option reflexively, by analogy:

    /// Equivalent to --mode name.
    #[arg(long, group="themode")]
    name: bool,

    /// Equivalent to --mode numeric.
    ///
    /// For --zodiac, show the ecliptic longitude from 0 to 360° decimal.
    #[arg(short, long, group="themode")]
    numeric: bool,

    /// Equivalent to --mode emoji
    #[arg(short, long, group="themode")]
    emoji: bool,

    /// Equivalent to --mode nerd-emoji
    #[arg(short, long="nerd-emoji", group="themode", short='N')]
    nerd_emoji: bool,

    /// Instead of displaying the moon phase, show the lunar zodiac sign.
    #[arg(short, long)]
    zodiac: bool,

    /// Use emojis direction for the Southern hemisphere (waxing crescent is 🌘)
    #[arg(short, long)]
    south_hemisphere: bool,

    /// Use variation selectors to prefer colour emoji (support depends on fonts/terminal)
    #[arg(short, long, group="vs")]
    color_emoji: bool,

    /// Use variation selectors to prefer text emoji (monochrome)
    #[arg(short, long, group="vs")]
    text_emoji: bool,

    /// Use cartoon face moon emojis (reduce distinct phases from 8 to 4).
    ///
    /// For zodiac signs, use cartoon animals and fun symbols.
    #[arg(short, long)]
    face_emoji: bool,


    /// Date with optional time to query the moon phase
    /// (e.g. "2023-10-31", "2023-10-31 23:59:59", "Friday").
    /// By default, show the current date and time.
    date: Option<String>,

}

fn str_to_system_time(timestr: &str) -> Result<SystemTime, &'static str> {
    match from_human_time(timestr) {
        Ok(result) => {
            match result {
                human_date_parser::ParseResult::DateTime(dt) => {
                    let utc: DateTime<Utc> = dt.into();
                    Ok(utc.into())
                },
                human_date_parser::ParseResult::Date(nd) => {
                    // can you get the local tz without needing a .now()?
                    let tz = chrono::Local::now().timezone();
                    let datetime_local = tz.with_ymd_and_hms(
                        nd.year(), nd.month(), nd.day(),
                        0,0,0
                    );
                    let datetime_utc: DateTime<Utc> = datetime_local.unwrap().into();
                    Ok(datetime_utc.into())
                },
                human_date_parser::ParseResult::Time(nt) => {
                    let now = chrono::Local::now();
                    let tz = now.timezone();
                    let datetime_local = tz.with_ymd_and_hms(
                        now.year(), now.month(), now.day(),
                        nt.hour(), nt.minute(), nt.second(),
                    );
                    let datetime_utc: DateTime<Utc> = datetime_local.unwrap().into();
                    Ok(datetime_utc.into())
                }
            }
        }
        Err(_) => Err("Invalid date")
    }
}

fn emoji_with_vs(one_emoji_char: &str, vari: EmojiVariation) -> String {
        let vs = match vari {
            EmojiVariation::Text => VS15,
            EmojiVariation::Colour => VS16,
            EmojiVariation::Unspecified => ""
        };
        format!("{}{}", one_emoji_char, vs)
}

fn to_emoji(phase: f64,
            south_hemisphere: bool,
            face: bool,
            vari: EmojiVariation)
    -> String {
        let emoji_set = if south_hemisphere && face {
            SOUTH_EMOJI_FACE
        } else if south_hemisphere {
            SOUTH_EMOJI
        } else if face {
            NORTH_EMOJI_FACE
        } else {
            NORTH_EMOJI
        };
        let emoji = match phase {
            x if x <  0.125 => emoji_set[0],
            x if x <  0.25  => emoji_set[1],
            x if x <  0.375 => emoji_set[2],
            x if x <  0.50  => emoji_set[3],
            x if x <  0.625 => emoji_set[4],
            x if x <  0.75  => emoji_set[5],
            x if x <  0.875 => emoji_set[6],
            x if x <  1.00  => emoji_set[7],
            _ => emoji_set[0]
        };

        emoji_with_vs(emoji, vari)
}

fn to_nerd_emoji(phase: f64,
            south_hemisphere: bool,
            vari: EmojiVariation)
    -> String {
        let emoji_set = if south_hemisphere {
            SOUTH_NERD_EMOJI
        } else {
            NORTH_NERD_EMOJI
        };
        let emoji = match phase {
            x if x <  0.035714286 => emoji_set[0],
            x if x <  0.071428571 => emoji_set[1],
            x if x <  0.10714286  => emoji_set[2],
            x if x <  0.14285714  => emoji_set[3],
            x if x <  0.17857143  => emoji_set[4],
            x if x <  0.21428571  => emoji_set[5],
            x if x <  0.25        => emoji_set[6],
            x if x <  0.28571429  => emoji_set[7],
            x if x <  0.32142857  => emoji_set[8],
            x if x <  0.35714286  => emoji_set[9],
            x if x <  0.39285714  => emoji_set[10],
            x if x <  0.42857143  => emoji_set[11],
            x if x <  0.46428571  => emoji_set[12],
            x if x <  0.5         => emoji_set[13],
            x if x <  0.51724138  => emoji_set[14],
            x if x <  0.55172414  => emoji_set[15],
            x if x <  0.5862069   => emoji_set[16],
            x if x <  0.62068966  => emoji_set[17],
            x if x <  0.65517241  => emoji_set[18],
            x if x <  0.68965517  => emoji_set[19],
            x if x <  0.72413793  => emoji_set[20],
            x if x <  0.75862069  => emoji_set[21],
            x if x <  0.79310345  => emoji_set[22],
            x if x <  0.82758621  => emoji_set[23],
            x if x <  0.86206897  => emoji_set[24],
            x if x <  0.89655172  => emoji_set[25],
            x if x <  0.93103448  => emoji_set[26],
            x if x <  0.96551724  => emoji_set[27],
            _ => emoji_set[0]
        };

        emoji_with_vs(emoji, vari)
}

fn main() {
    let cli = Cli::parse();

    let mode = if cli.numeric {
        Mode::Numeric
    } else if cli.emoji {
        Mode::Emoji
    } else if cli.name {
        Mode::Name
    } else if cli.nerd_emoji {
        Mode::NerdEmoji
    } else if cli.face_emoji || cli.color_emoji || cli.text_emoji {
        // if user is setting emoji options, it implies they want emoji mode.
        Mode::Emoji
    } else {
        cli.mode // default is Mode::Name
    };

    let emoji_variation = match mode {
        Mode::Emoji => {
            if cli.text_emoji { EmojiVariation::Text }
            else if cli.color_emoji { EmojiVariation::Colour }
            else { EmojiVariation::Unspecified }
        },
        _ => EmojiVariation::Unspecified
    };

    let moontime: SystemTime;
    if cli.date.is_some() {
        match str_to_system_time(cli.date.unwrap().as_str()) {
            Ok(t) => { moontime = t;}
            Err(_) => {
                println!("Invalid date!");
                std::process::exit(2);
            }
        }
    } else {
        moontime = SystemTime::now();
    }

    let moon = MoonPhase::new(moontime);

    if cli.zodiac {
        match mode {
            Mode::Name  => println!("{}", moon.zodiac_name),
            Mode::Numeric => {
                println!("{:1.2}", moon.longitude);
            },
            Mode::Emoji => {
                let emoji = if cli.face_emoji {
                    match moon.zodiac_name {
                        "Pisces"=> "🐟",
                        "Aries"=> "🐏",
                        "Taurus"=> "🐂",
                        "Gemini"=> "👯",
                        "Cancer"=> "🦀",
                        "Leo"=> "🦁",
                        "Virgo"=> "👧",
                        "Libra"=> "⚖️",
                        "Scorpio"=> "🦂",
                        "Sagittarius"=> "🏹",
                        "Capricorn"=> "🐐",
                        "Aquarius"=> "🏺",
                        _ => "🐍",
                    }
                } else {
                    match moon.zodiac_name {
                        "Pisces"=> "♓",
                        "Aries"=> "♈",
                        "Taurus"=> "♉",
                        "Gemini"=> "♊",
                        "Cancer"=> "♋",
                        "Leo"=> "♌",
                        "Virgo"=> "♍",
                        "Libra"=> "♎",
                        "Scorpio"=> "♏",
                        "Sagittarius"=> "♐",
                        "Capricorn"=> "♑",
                        "Aquarius"=> "♒",
                        _ => "⛎",
                    }
                };
                println!("{}", emoji_with_vs(emoji, emoji_variation));
            }
            Mode::NerdEmoji => {
                let nerd_emoji = to_nerd_emoji(moon.phase,
                                     cli.south_hemisphere,
                                     emoji_variation);

                println!("{}", nerd_emoji);
            },
        };
    } else {
        match mode {
            Mode::Numeric   => println!("{:1.2}", moon.phase),
            Mode::Name      => println!("{}", moon.phase_name),
            Mode::NerdEmoji => {
                let nerd_emoji = to_nerd_emoji(moon.phase,
                                     cli.south_hemisphere,
                                     emoji_variation);

                println!("{}", nerd_emoji);
            },
            Mode::Emoji     => {
                let emoji = to_emoji(moon.phase,
                                     cli.south_hemisphere,
                                     cli.face_emoji,
                                     emoji_variation);

                println!("{}", emoji);
            }
        }
    }
}
