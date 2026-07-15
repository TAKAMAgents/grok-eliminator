use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "grok-eliminator",
    version,
    about = "Audit or remove local Grok CLI artifacts"
)]
struct Cli {
    #[arg(long, global = true, help = "Render a machine-readable JSON report")]
    json: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Audit,
    Remove {
        #[arg(long, help = "Apply filesystem and environment mutations")]
        apply: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    let home = match grok_eliminator::HomeDirectory::current() {
        Ok(home) => home,
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(2);
        }
    };
    let engine = grok_eliminator::CleanupEngine::new(home);
    let report = match cli.command {
        Command::Audit => engine.audit(),
        Command::Remove { apply: false } => engine.plan_removal(),
        Command::Remove { apply: true } => engine.apply_removal(),
    };

    if cli.json {
        match serde_json::to_string_pretty(&report) {
            Ok(output) => println!("{output}"),
            Err(error) => {
                eprintln!("error: failed to encode report: {error}");
                std::process::exit(1);
            }
        }
    } else {
        println!("{}", report.render_text());
    }
}
