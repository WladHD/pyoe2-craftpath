use clap::{Parser, Subcommand};

use craftpath_server::{cli, config, rest, worker};
#[cfg(feature = "mcp")]
use craftpath_server::mcp;

#[derive(Parser, Debug)]
#[command(
    name = "pyoe2-backend",
    author = "Wladislaw Jerokin (WladHD)",
    version,
    about = "pyoe2-craftpath backend",
    long_about = "Backend services for pyoe2-craftpath. Runs as a REST API node, a calculation worker, an MCP server or the classic interactive CLI."
)]
struct Args {
    #[command(subcommand)]
    mode: Mode,
}

#[derive(Subcommand, Debug)]
enum Mode {
    /// Serve the REST + WebSocket API (enqueues jobs, never computes).
    Rest,
    /// Consume calculation jobs from the Redis queue.
    Worker,
    /// Serve the Model Context Protocol endpoint for LLM clients.
    Mcp(McpArgs),
    /// Run a single calculation interactively (the classic CLI).
    Cli(cli::CliArgs),
}

#[derive(clap::Args, Debug)]
struct McpArgs {
    /// Transport: "http" (streamable HTTP, default; for k8s) or "stdio"
    /// (for local clients like Claude Desktop / Claude Code).
    #[arg(long, default_value = "http")]
    transport: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.mode {
        Mode::Cli(cli_args) => {
            cli::run_cli(cli_args);
            Ok(())
        }
        mode => {
            craftpath_core::utils::logger_utils::init_tracing();
            let config = config::Config::from_env()?;

            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?;

            runtime.block_on(async move {
                match mode {
                    Mode::Rest => rest::serve(config).await,
                    Mode::Worker => worker::run(config).await,
                    #[cfg(feature = "mcp")]
                    Mode::Mcp(mcp_args) => mcp::serve(config, &mcp_args.transport).await,
                    #[cfg(not(feature = "mcp"))]
                    Mode::Mcp(_) => anyhow::bail!(
                        "this binary was built without the 'mcp' feature"
                    ),
                    Mode::Cli(_) => unreachable!(),
                }
            })
        }
    }
}
