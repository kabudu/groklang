use clap::{Parser as ClapParser, Subcommand};
use std::fs;

mod lexer;
mod ast;
mod parser;
mod type_checker;
mod ir;
mod vm;
mod ai;
mod lsp;

use lexer::Lexer;
use parser::Parser as GrokParser;
use type_checker::TypeChecker;
use ir::IRGenerator;
use vm::VM;

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
            println!("Parsed AST: {:?}", ast);

            let mut type_checker = TypeChecker::new();
            type_checker.check(&ast)?;
            println!("Type check passed");

            let ir_gen = IRGenerator::new();
            let ir_functions = ir_gen.generate(&ast);
            println!("Generated IR: {:?}", ir_functions);

            let mut vm = VM::new();
            vm.load_program(&ir_functions);
            if !ir_functions.is_empty() {
                vm.execute(&ir_functions[0].name)?;
            }
            println!("Execution completed");
            Ok(())
        }
        Commands::Lsp => {
            lsp::run_lsp().await
        }
    }
}
