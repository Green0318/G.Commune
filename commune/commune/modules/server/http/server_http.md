<div align="center">

# **Commune AI**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Discord Chat](https://img.shields.io/badge/discord-join%20chat-blue.svg)](https://discord.com/invite/DgjvQXvhqf)
[![Website Uptime](https://img.shields.io/website-up-down-green-red/http/monip.org.svg)](https://www.communeai.org/)
[![Twitter Follow](https://img.shields.io/twitter/follow/communeaidotorg.svg?style=social&label=Follow)](https://twitter.com/communeaidotorg)

### An Open Modules Network

</div>

# Convert a Python module forwarding incoming request into rust

## Benefits

- Performance: Rust is known for its performance and can be much faster than Python due to its emphasis on zero-cost abstractions.

- Concurrency: Rust's ownership model and safety guarantees allow for safe concurrent programming, making it easier to handle multiple requests efficiently with threads.

### Convert python code to rust.

Core feature of Server is function forward which is wrapping incoming request.

```bash
def forward(self, fn:str, input:dict):        
        user_info = None
        try:
            input['fn'] = fn
            # you can verify the input with the server key class
            if not self.public:
                assert self.key.verify(input), f"Data not signed with correct key"

            ...

            if self.save_history:

            output = {
            'module': self.name,
            'fn': fn,
            'timestamp': input['data']['timestamp'],
            'address': input['address'],
            'args': input['data']['args'],
            'kwargs': input['data']['kwargs'],
            'result': None if self.sse else result,
            'user': user_info,

            }
            output.update(output.pop('data', {}))
            output['latency'] = c.time() - output['timestamp']
            self.add_history(output)

        return result

            ...
    
```

# How to

Core feature of Server is function forward which is wrapping incoming request.

## First, how to create rust module and use this in python file.

### For starters, we need to:
  - bring in the pyo3 prelude
  - annotate the function with #[pyfunction] to turn it into a PyCFunction
  - wrap the result in a PyResult
  - add the function to the #[pymodule]

To import the rust module in the python code, compiled the rust code into a shared library that can be loaded by python.

## Second, how to call python module in rust code.

Ultimate object of this function is to call fn_obj which is python function.
But fn_obj can be random functions of C modules.
So the only way is to optimize around the forward.
Need to use this python module in the rust code.

```bash

...

if callable(fn_obj):
    result = fn_obj(*args, **kwargs)
else:
    result = fn_obj

...

```
This is the part that must be placed to the lib.rs

```bash

...

#[pyfunction]
async fn forward_worker(obj: &PyAny, fn_name: &str, input: &PyDict,sender: Sender<Result<Value, Box<dyn Error>>>){
    let mut user_info: Option<PyObject> = None;
...

#[tokio::main]
fn forward(py: Python, obj: &PyAny, fn_name: &str, input: &PyDict) -> PyResult<PyObject> {
    let (tx, rx) = mpsc::channel(1); // Channel for sending result

...

#[pymodule]
fn My_module(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(forward, m)?)?;
    Ok(())
}

```

This is the part that must be attached to the Server_http.py

```bash

...

import My_module

result = My_module.forward( ... )

...

```

# Developement FAQ

- Where can i find futher documentation? This repository folder, [Doc](https://github.com/commune-ai/commune/tree/main/docs).
- Can I install on Windows? Yes, [Guide](https://github.com/OmnipotentLabs/communeaisetup).
- Can I contribute? Absolutely! We are open to all contributions. Please feel free to submit a pull request.