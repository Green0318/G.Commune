<div align="center">

# **Commune AI**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Discord Chat](https://img.shields.io/badge/discord-join%20chat-blue.svg)](https://discord.com/invite/DgjvQXvhqf)
[![Website Uptime](https://img.shields.io/website-up-down-green-red/http/monip.org.svg)](https://www.communeai.org/)
[![Twitter Follow](https://img.shields.io/twitter/follow/communeaidotorg.svg?style=social&label=Follow)](https://twitter.com/communeaidotorg)

### An Open Modules Network

</div>

# Benefits for converting a Python module that starts a client session and sends requests into Rust

- Performance: Rust is known for its performance and can be much faster than Python due to its emphasis on zero-cost abstractions.

- Concurrency: Rust's ownership model and safety guarantees allow for safe concurrent programming, making it easier to handle multiple requests efficiently with threads.

### Convert python code to rust.

```bash
async with aiohttp.ClientSession() as session:
            async with session.post(url, json=request, headers=headers) as response:

            ...

            else:
                raise ValueError(f"Invalid response content type: {response.content_type}")
```

# How to

Two challenges to be solved to improve performance and support concurrency.

## First,how to create rust module and use this in python file.

Created rust module which generates a thread whenever client requests and send http result to each request in that thread.


To import the rust module in the python code, compiled the rust code into a shared library that can be loaded by python.


## Second, how to call python module in rust code.

Need to use this python module in the rust code.

```bash

if self.debug:
    progress_bar = c.tqdm(desc='MB per Second', position=0)

```

```bash

let gil = Python::acquire_gil();
let py = gil.python();
// Import the tqdm module and get the tqdm function
let tqdm = py.import("tqdm")?.getattr("tqdm")?;
// Call tqdm function with desired arguments
let progress_bar = tqdm.call1(("MB per Second",0))?; 

...

progress_bar.call_method1("update", (event_bytes / BYTES_PER_MB,))?;

```

# Developement FAQ

- Where can i find futher documentation? This repository folder, [Doc](https://github.com/commune-ai/commune/tree/main/docs).
- Can I install on Windows? Yes, [Guide](https://github.com/OmnipotentLabs/communeaisetup).
- Can I contribute? Absolutely! We are open to all contributions. Please feel free to submit a pull request.
