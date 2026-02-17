use clap::{Parser as ClapParser, Subcommand};
use std::fs;

use grok::borrow_checker::BorrowChecker;
use grok::ir::IRGenerator;
use grok::lsp;
use grok::macro_expander::MacroExpander;
use grok::parser::Parser as GrokParser;
use grok::type_checker::TypeChecker;
use grok::vm::VM;

#[derive(ClapParser)]
#[command(name = "grok")]
#[command(about = "GrokLang compiler")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check a GrokLang file (parse, macro, type, borrow checks)
    Check {
        /// Input file
        file: String,
    },
    /// Compile a GrokLang file
    Compile {
        /// Input file
        file: String,
        /// Execute after compile
        #[arg(long)]
        run: bool,
        /// Entrypoint function name for execution
        #[arg(long, default_value = "main")]
        entry: String,
    },
    /// Run the LSP server
    Lsp,
}

fn run_frontend_pipeline(
    source: &str,
) -> Result<(grok::ast::AstNode, Vec<grok::ir::IRFunction>), String> {
    let parser = GrokParser::new();
    let ast = parser.parse(source)?;

    let mut expander = MacroExpander::new();
    let ast = expander.expand(ast);

    let mut type_checker = TypeChecker::new();
    type_checker.check(&ast)?;

    let mut borrow_checker = BorrowChecker::new();
    borrow_checker.check(&ast)?;

    let mut ir_gen = IRGenerator::new();
    let ir_functions = ir_gen.generate(&ast);

    Ok((ast, ir_functions))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();

    match args.command {
        Commands::Check { file } => {
            let source = fs::read_to_string(&file)?;
            let (_, ir_functions) =
                run_frontend_pipeline(&source).map_err(std::io::Error::other)?;

            println!(
                "Check passed: {} ({} IR function(s))",
                file,
                ir_functions.len()
            );
            Ok(())
        }
        Commands::Compile { file, run, entry } => {
            let source = fs::read_to_string(&file)?;
            let (_, ir_functions) =
                run_frontend_pipeline(&source).map_err(std::io::Error::other)?;

            println!("Compiled {} function(s) from {}", ir_functions.len(), file);

            if run {
                if !ir_functions.iter().any(|f| f.name == entry) {
                    return Err(std::io::Error::other(format!(
                        "Entrypoint '{}' not found. Available functions: {}",
                        entry,
                        ir_functions
                            .iter()
                            .map(|f| f.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ))
                    .into());
                }

                let mut vm = VM::new();
                vm.load_program(&ir_functions);
                let result = vm.execute(entry.clone(), None).await?;
                println!("Execution result ({}): {:?}", entry, result);
            }
            Ok(())
        }
        Commands::Lsp => lsp::run_lsp().await,
    }
}
