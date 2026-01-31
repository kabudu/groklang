use clap::{Parser as ClapParser, Subcommand};
use std::fs;

use grok::ir::IRGenerator;
use grok::macro_expander::MacroExpander;
use grok::parser::Parser as GrokParser;
use grok::type_checker::TypeChecker;
use grok::vm::VM;
use grok::lsp;

#[derive(ClapParser)]
#[command(name = "grok")]
#[command(about = "GrokLang compiler")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a GrokLang file
    Compile {
        /// Input file
        file: String,
    },
    /// Run the LSP server
    Lsp,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();

    match args.command {
        Commands::Compile { file } => {
            let source = fs::read_to_string(&file)?;
            let parser = GrokParser::new();
            let ast = parser.parse(&source)?;

            let mut expander = MacroExpander::new();
            let ast = expander.expand(ast);

            println!("Expanded AST: {:?}", ast);

            let mut type_checker = TypeChecker::new();
            match type_checker.check(&ast) {
                Ok(substitutions) => {
                    println!("Type check passed. Inferred types:");
                    for (var, ty) in substitutions {
                        println!("  {}: {:?}", var, ty);
                    }
                }
                Err(e) => {
                    eprintln!("Type error: {}", e);
                    std::process::exit(1);
                }
            }

            let mut ir_gen = IRGenerator::new();
            let ir_functions = ir_gen.generate(&ast);
            println!("Generated IR: {:?}", ir_functions);

            let mut vm = VM::new();
            vm.load_program(&ir_functions);
            if !ir_functions.is_empty() {
                let result = vm.execute(ir_functions[0].name.clone(), None).await?;
                println!("Execution result: {:?}", result);
            }
            println!("Execution completed");
            Ok(())
        }
        Commands::Lsp => lsp::run_lsp().await,
    }
}
