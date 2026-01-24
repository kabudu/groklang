use crate::ir::IRGenerator;
use crate::parser::Parser;
use crate::vm::VM;
use pyo3::prelude::*;
use std::sync::{Arc, Mutex};

#[pyclass]
struct GrokInterpreter {
    vm: Arc<Mutex<VM>>,
}

#[pymethods]
impl GrokInterpreter {
    #[new]
    fn new() -> Self {
        GrokInterpreter {
            vm: Arc::new(Mutex::new(VM::new())),
        }
    }

    fn run(&self, code: String) -> PyResult<String> {
        let parser = Parser::new();
        let ast = match parser.parse(&code) {
            Ok(a) => a,
            Err(e) => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(e)),
        };

        let mut gen = IRGenerator::new();
        let ir = gen.generate(&ast);

        // We need to create a runtime for the async VM execution
        let rt = tokio::runtime::Runtime::new().unwrap();

        let result = rt.block_on(async {
            // Note: In a real FFI scenario, reuse the VM state correctly.
            // For this basic binding, we reload and run.
            // A more complex implementation would separate compilation and execution steps.

            // To be safe with Arc<Mutex<VM>>, we'd need to lock, but VM::execute consumes self in our current design.
            // To support FFI properly, we'd need to refactor VM::execute to take &mut self.
            // For now, let's just clone a fresh one for the demo, or modify VM to be FFI friendly.

            // Temporary workaround: Create a new VM instance for this run, ignoring persisted state for now.
            // This is "stateless" execution from Python's perspective.
            let mut vm = VM::new();
            vm.load_program(&ir);
            vm.execute("main".to_string(), None).await
        });

        match result {
            Ok(val) => Ok(format!("{:?}", val)),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e)),
        }
    }
}

#[pymodule]
#[pyo3(name = "grok")]
fn grok(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<GrokInterpreter>()?;
    Ok(())
}
