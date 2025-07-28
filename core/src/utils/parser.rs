use clap::Parser;
use clap::{Args, FromArgMatches, Subcommand, ValueEnum};
use clap_verbosity_flag::{InfoLevel, Verbosity};

#[derive(Parser, Debug, Clone)]
#[clap(author = "Dawid Sieluzycki @Leghart", version = "0.1.0", about)]
struct RawArgs {
    #[command(subcommand)]
    subparser: Option<Subparser>,

    #[arg(short, long, default_value_t = Mode::Game)]
    pub mode: Mode,

    #[clap(flatten)]
    pub verbose: Verbosity<InfoLevel>,
}

#[derive(Debug, Clone, Subcommand)]
enum Subparser {
    Monitor(ReClap<MonitorArgs, Self>),
    Stockfish(ReClap<StockfishArgs, Self>),
    Imgproc(ReClap<ImgProcArgs, Self>),
    Engine(ReClap<EngineArgs, Self>),
}

#[derive(Debug, Clone, Parser)]
pub struct MonitorArgs {
    #[arg(short, long, default_value_t = 0)]
    pub number: u8,
}

#[derive(Debug, Clone, Parser)]
pub struct StockfishArgs {
    #[arg(
        short,
        long,
        default_value = "/home/leghart/projects/cheatess/stockfish-ubuntu-x86-64-avx2"
    )]
    pub path: std::path::PathBuf,

    #[arg(short, long, default_value_t = 1700)]
    pub elo: usize,

    #[arg(short, long, default_value_t = 20)]
    pub skill: u8,

    #[arg(short, long, default_value_t = 5)]
    pub depth: u8,

    #[arg(long, default_value_t = 16)]
    pub hash: usize,
}

#[derive(Debug, Clone, Parser)]
pub struct ImgProcArgs {
    #[arg(short, long, default_value_t = 5)]
    pub margin: u8,

    #[arg(short, long, default_value_t = 0.1)]
    pub piece_threshold: f64,

    #[arg(short, long, default_value_t = 127.0)]
    pub extract_piece_threshold: f64,

    #[arg(short, long, default_value_t = 100.0)]
    pub board_threshold: f64,

    #[arg(short, long, default_value_t = 500)]
    pub difference_level: i32,
}

#[derive(Debug, Clone, Parser)]
pub struct EngineArgs {
    #[arg(short, long, default_value_t = false)]
    pub pretty: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum, Default)]
pub enum Mode {
    #[default]
    Game,
    Test,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Mode::Game => "game",
            Mode::Test => "test",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug)]
pub struct CheatessArgs {
    pub verbose: Verbosity<InfoLevel>,
    pub mode: Mode,
    pub monitor: MonitorArgs,
    pub stockfish: StockfishArgs,
    pub proc_image: ImgProcArgs,
    pub engine: EngineArgs,
}

pub fn parse_args_from<I: IntoIterator<Item = T>, T: Into<String>>(iterator: I) -> CheatessArgs {
    let mut updated: Vec<String> = iterator.into_iter().map(Into::into).collect();

    for subparser in ["monitor", "stockfish", "imgproc", "engine"] {
        if !updated.contains(&subparser.to_string()) {
            updated.push(subparser.to_string());
        }
    }

    let args = RawArgs::parse_from(updated);
    let mut monitor: Option<MonitorArgs> = None;
    let mut stockfish: Option<StockfishArgs> = None;
    let mut proc_image: Option<ImgProcArgs> = None;
    let mut engine: Option<EngineArgs> = None;

    let mut next = args.subparser;
    while let Some(sub) = next {
        next = match sub {
            Subparser::Monitor(rec) => {
                monitor = Some(rec.inner);
                (rec.next).map(|d| *d)
            }
            Subparser::Stockfish(rec) => {
                stockfish = Some(rec.inner);
                (rec.next).map(|d| *d)
            }
            Subparser::Imgproc(rec) => {
                proc_image = Some(rec.inner);
                (rec.next).map(|d| *d)
            }
            Subparser::Engine(rec) => {
                engine = Some(rec.inner);
                (rec.next).map(|d| *d)
            }
        }
    }

    CheatessArgs {
        monitor: monitor.expect("Monitor hasn't been extracted"),
        stockfish: stockfish.expect("Stockfish hasn't been extracted"),
        proc_image: proc_image.expect("ImgProc hasn't been extracted"),
        engine: engine.expect("Engine hasn't been extracted"),
        verbose: args.verbose,
        mode: args.mode,
    }
}

// Implementation for many subcommands for clap
// https://github.com/clap-rs/clap/issues/2222#issuecomment-2524152894

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct ReClap<T, U>
where
    T: Args,
    U: Subcommand,
{
    pub inner: T,
    pub next: Option<Box<U>>,
}

impl<T, U> Args for ReClap<T, U>
where
    T: Args,
    U: Subcommand,
{
    fn augment_args(cmd: clap::Command) -> clap::Command {
        T::augment_args(cmd).defer(|cmd| U::augment_subcommands(cmd.disable_help_subcommand(true)))
    }
    fn augment_args_for_update(_cmd: clap::Command) -> clap::Command {
        unimplemented!()
    }
}

impl<T, U> FromArgMatches for ReClap<T, U>
where
    T: Args,
    U: Subcommand,
{
    fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
        let inner = T::from_arg_matches(matches)?;
        let next = if let Some((_name, _sub)) = matches.subcommand() {
            Some(U::from_arg_matches(matches)?)
        } else {
            None
        };
        Ok(Self {
            inner,
            next: next.map(Box::new),
        })
    }
    fn update_from_arg_matches(&mut self, _matches: &clap::ArgMatches) -> Result<(), clap::Error> {
        unimplemented!()
    }
}
