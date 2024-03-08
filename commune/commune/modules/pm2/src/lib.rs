use pyo3::prelude::*;
use steward::prelude::*;

#[macro_use]
extern crate steward;

use steward::{Cmd, Env, ProcessPool, Process};

#[tokio::main]
async fn main(cmd_command: String) -> steward::Result<()> {
    execeteCmd::execute(cmd_command).run().await?;
    Ok(())
}
mod execeteCmd{
    fn execute(cmd_command: String) -> Cmd {
        cmd! {
          exe: cmd_command,
          env: Env::empty(),
          pwd: Loc::root(),
          msg: "executing cmd",
        }
    }
}
#[pymodule]
fn rust(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(main, m)?)?;
    Ok(())
}
