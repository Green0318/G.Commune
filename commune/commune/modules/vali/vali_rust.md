<div align="center">

# **Commune AI**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Discord Chat](https://img.shields.io/badge/discord-join%20chat-blue.svg)](https://discord.com/invite/DgjvQXvhqf)
[![Website Uptime](https://img.shields.io/website-up-down-green-red/http/monip.org.svg)](https://www.communeai.org/)
[![Twitter Follow](https://img.shields.io/twitter/follow/communeaidotorg.svg?style=social&label=Follow)](https://twitter.com/communeaidotorg)

### An Open Modules Network

</div>

Vali has many functions such as validation, staking, registration and etc.

## Improve the performance of Vali using PyO3.

### To improve this performance, converted the python module's main functions into rust using PyO3.
PyO3 uses Rust's "procedural macros" to provide a powerful yet simple API to denote what Rust code should map into Python objects.
The PyO3 project lets you leverage the best of both worlds by writing Python extensions in Rust. With PyO3, you write Rust code, indicate how it interfaces with Python, then compile Rust and deploy it directly into a Python virtual environment, where you can use it unobtrusively with your Python code.

## How to

 Three most important Steps to be solved when using PyO3

### Calling Python from Rust.

#### Python object types available in PyO3's API

- How to work with Python exceptions
- How to call Python functions
- How to execute existing Python code

### Using Rust from Python

- Python modules, via the #[pymodule] macro
- Python functions, via the #[pyfunction] macro
- Python classes, via the #[pyclass] macro (plus #[pymethods] to define methods for those clases)

### Install Rust Library as a Python module using Maturin build tool.

```bash
root@abcb1493868e:/commune/commune/vali# python3 -m venv .env
root@abcb1493868e:/commune/commune/vali# source .env/bin/activate
(.env) root@abcb1493868e:/commune/commune/vali# pip install maturin
Collecting maturin
    ...
Installing collected packages: toml, maturin
Successfully installed maturin-0.11.5 toml-0.10.2
(.env) root@abcb1493868e:/commune/commune/vali# maturin develop
🔗 Found vali bindings
🐍 Found CPython 3.9 at python
   Compiling proc-macro2 v1.0.32
    ...
   Compiling vali v0.1.0 (/commune/vali)
    Finished dev [unoptimized + debuginfo] target(s) in 23.48s
(.env) root@abcb1493868e:/commune# c vali/set_network
```

### The result

```bash

root@abcb1493868e:/commune# c vali/set_network
{'network': 'main', 'url': 'https://commune-api-node-1.communeai.net/'}
{'search': None, 'network': 'subspace', 'netuid': 0, 'n': 8141, 'msg': 'Synced network'}

```


# Developement FAQ

- Where can i find futher documentation? This repository folder, [Doc](https://github.com/commune-ai/commune/tree/main/docs).
- Can I install on Windows? Yes, [Guide](https://github.com/OmnipotentLabs/communeaisetup).
- Can I contribute? Absolutely! We are open to all contributions. Please feel free to submit a pull request.


