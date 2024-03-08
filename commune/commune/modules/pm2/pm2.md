<div align="center">

# **Commune AI**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Discord Chat](https://img.shields.io/badge/discord-join%20chat-blue.svg)](https://discord.com/invite/DgjvQXvhqf)
[![Website Uptime](https://img.shields.io/website-up-down-green-red/http/monip.org.svg)](https://www.communeai.org/)
[![Twitter Follow](https://img.shields.io/twitter/follow/communeaidotorg.svg?style=social&label=Follow)](https://twitter.com/communeaidotorg)

### An Open Modules Network

</div>

# Convert a Python module into Rust for process manager

## Benefits of steward

Steward: Task runner and process manager for Rust.
If you're not happy managing your infrastructure with a pile of bash scripts, steward might be helpful. 

### How to excute process manager

In the pm2 module, command cmd is used like this.

```bash

def cmd(cls, command:Union[str, list],
            *args,
                        verbose:bool = True, 
                        env:Dict[str, str] = {}, 
                        sudo:bool = False,
                        password: bool = None,
                        color: str = 'white',
                        bash : bool = False,
                        **kwargs):
        return c.module('os').cmd( 
                        command,
                        *args,
                        verbose=verbose, 
                        env= env,
                        sudo=sudo,
                        password=password,
                        color=color,
                        bash=bash,
                        **kwargs)

```

In order to improve the performance, Steward can be used.



```bash

c.cmd(f"pm2 restart {n}", verbose=False)

...

cls.cmd(f"pm2 delete {n}", verbose=False)

```

# How to

Two challenges to be solved to improve performance and support concurrency.

## First, how to create rust module using steward

In the lib.rs file,

```bash
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
```
And in the Cargo.toml file,

```bash

...
[dependencies]
steward = "0.0.5"
...

```

## Second, how to call rust module in python code.

Need to use this rust module in the python code.
Maturin can be used to carry out this matter

```bash

...
puppy:/pyo3/pm2# python3 -m venv .env
puppy:/pyo3/pm2# source .env/bin/activate
(.env) puppy:/pyo3/pm2# pip install maturin
Collecting maturin
    ...
Installing collected packages: toml, maturin
Successfully installed maturin-0.11.5 toml-0.10.2
(.env) puppy:/pyo3/pm2# maturin develop
🔗 Found pyo3 bindings
🐍 Found CPython 3.9 at python
   Compiling proc-macro2 v1.0.32
    ...
   Compiling pm2 v0.1.0 (/pyo3/pm2)
    Finished dev [unoptimized + debuginfo] target(s) in 23.48s
(.env) puppy:/pyo3/pm2# python3 pm2.py
...

```

Then, import this rust module in the python code.



# Developement FAQ

- Where can i find futher information about steward? This repository folder, [Doc](https://docs.rs/steward/latest/steward/).
- Where can i know about maturin? Here, [Doc](https://saidvandeklundert.net/learn/2021-11-18-calling-rust-from-python-using-pyo3/).
- Can I install on Windows? Yes, [Guide](https://github.com/OmnipotentLabs/communeaisetup).
- Can I contribute? Absolutely! We are open to all contributions. Please feel free to submit a pull request.

