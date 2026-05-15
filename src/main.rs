mod lib;

use clap::{Parser, Subcommand};
use lib::{convert_format, run_config_conversion, Format};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "FEX - Format EXchange")] 
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short = 'f', long = "from", value_enum, value_name = "FORMAT")]
    from_format: Option<Format>,

    #[arg(short = 't', long = "to", value_enum, value_name = "FORMAT")]
    to_format: Option<Format>,

    #[arg(short = 'i', long = "input", value_name = "INPUT")]
    input: Option<PathBuf>,

    #[arg(short = 'o', long = "output", value_name = "OUTPUT")]
    output: Option<PathBuf>,

    #[arg(long = "do-not-keep-old-file")]
    do_not_keep_old_file: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Config(ConfigArgs),
}

#[derive(Parser, Debug)]
struct ConfigArgs {
    #[arg(long = "use", value_name = "FILE")]
    config: PathBuf,

    #[arg(short = 'i', long = "input", value_name = "INPUT")]
    input: PathBuf,

    #[arg(short = 'o', long = "output", value_name = "OUTPUT")]
    output: PathBuf,

    #[arg(long = "it-is-convert", conflicts_with = "it_is_converted")]
    it_is_convert: bool,

    #[arg(long = "it-is-converted", conflicts_with = "it_is_convert")]
    it_is_converted: bool,

    #[arg(long = "do-not-keep-old-file")]
    do_not_keep_old_file: bool,
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Some(Commands::Config(config_args)) => run_config_command(config_args),
        None => run_convert_command(cli),
    };
    if let Err(error) = result {
        eprintln!("Error: {:#}", error);
        std::process::exit(1);
    }
}

fn run_convert_command(cli: Cli) -> anyhow::Result<()> {
    let from = cli.from_format.context("Missing required option: --from <FORMAT>")?;
    let to = cli.to_format.context("Missing required option: --to <FORMAT>")?;
    let input = cli.input.context("Missing required option: --input <INPUT>")?;
    let output = cli.output.context("Missing required option: --output <OUTPUT>")?;

    convert_format(from, to, &input, &output, cli.do_not_keep_old_file)
}

fn run_config_command(args: ConfigArgs) -> anyhow::Result<()> {
    if !args.it_is_convert && !args.it_is_converted {
        anyhow::bail!("Config mode requires either --it-is-convert or --it-is-converted");
    }
    run_config_conversion(
        &args.config,
        &args.input,
        &args.output,
        args.it_is_convert,
        args.do_not_keep_old_file,
    )
}
