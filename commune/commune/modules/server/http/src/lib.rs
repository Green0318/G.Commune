use pyo3::prelude::*;
use reqwest::Client;
use serde_json::Value;
use std::error::Error;
use tokio::sync::mpsc::{self, Sender};

#[pyfunction]
async fn forward_worker(obj: &PyAny, fn_name: &str, input: &PyDict,sender: Sender<Result<Value, Box<dyn Error>>>){
    let mut user_info: Option<PyObject> = None;

    // Wrap the code inside Python's with_gil to safely interact with Python objects
    Python::with_gil(|py| {
        let mut input: HashMap<&str, &str> = HashMap::new();
        input.insert("fn", fn_name);
        let isPublic = obj.getattr(public)?.extract::<bool>(py)?;
        // Verify the input with the server key class
        if !isPublic {
            // Assume `key.verify()` and `serializer.deserialize()` are Python methods
            let result = obj.getattr("key")?.call_method1("verify", (input,))?;
    
            // Extract the result as a boolean
            let is_verified: bool = result.extract(py)?;
        
            // Check if the verification passed
            if !is_verified {
                return Err(PyErr::new::<PyAssertionError, _>("Data not signed with correct key"));
            }
        }
        let args_exist = input.contains_key("args");
        let kwargs_exist = input.contains_key("kwargs");
        if args_exist && kwargs_exist {
            let mut data = PyDict::new(py);
            data.set_item("args", input.get_item("args"))?;
            data.set_item("kwargs", input.get_item("kwargs"))?;
            data.set_item("timestamp", input.get_item("timestamp"))?;
            data.set_item("address", input.get_item("address"))?;
            
            // Assign the 'data' dictionary to the 'data' key of the input dictionary
            input.set_item("data", data)?;
        }
        let deserialized = obj.getattr("serializer")?.call_method1("deserialize", (input.get_item("data"),))?;
        input.set_item("data", deserialized)?;
        let c = PyModule::import(py, "c")?;
        let timestamp_method = c.getattr("timestamp")?;

        // Execute the method and get the result
        let result: i64 = timestamp_method.call(())?.extract()?;



        let mut data: PyDict = input.get_item("data").unwrap().extract(py)?;

        // Verify the request is not too old
        let request_staleness: i64 = c.timestamp() - data.get_item("timestamp").unwrap().extract(py)?;
        if request_staleness >= obj.max_request_staleness {
            return Err(PyErr::new::<PyAssertionError, _>(format!("Request is too old, {} > MAX_STALENESS ({}) seconds old", request_staleness, obj.max_request_staleness)));
        }

        // Verify the access module
        user_info = obj.access_module.call_method1("verify", (input,))?;
        if user_info.get_item("passed").unwrap().extract::<bool>(py)? {
            return Ok(user_info);
        }

        let data_args: PyObject = data.get_item("args").unwrap().extract(py)?;
        let data_kwargs: PyObject = data.get_item("kwargs").unwrap().extract(py)?;
        let args: Vec<PyObject> = data_args.extract(py)?;
        let kwargs: HashMap<String, PyObject> = data_kwargs.extract(py)?;
        
        let fn_name : String = format!("{}::{}", self.name, function_name)
        let c = py.import("commune")?;
        let print: PyObject = c.getattr('print)?;
        print.call1(format!("🚀 Forwarding {} --> {} 🚀\033", input.get_item("address"), function_name),color :String = 'yellow');

        let fn_obj: PyObject = obj.module.getattr(fn_name)?;
        let fn_obj_callable: bool = fn_obj.hasattr("__call__")?;

        let result: PyObject;
        if fn_obj_callable {
            result = fn_obj.call(py, args, kwargs)?;
        } else {
            result = fn_obj;
        }
        let success: bool;

        if let Some(result) = result {
            if let Some(_) = result.get("error") {
                success = false;
            } else {
                success = true;
            }
        } else {
            success = true;
        }
        if success {
            let message = format!("✅ Success: {}::{} --> {}... ✅", name, function_name, address);
            println!("{}", message.green());
        } else {
            let message = format!("🚨 Error: {}::{} --> {}... 🚨", name, function_name, address);
            println!("{}", message.red());
        }
        let process_result: PyObject = obj.getattr('process_result')?;
        let save_history: bool = obj.getattr('save_history');
        if save_history{
            let mut output: HashMap<&str, &str> = HashMap::new();
            let name = "MyClass";
            let function_name = "my_function";
            let timestamp = "1234567890";
            let address = "example.com";
            let args = "arg1, arg2, arg3";
            let kwargs = "key1=value1, key2=value2";
            let result: Option<&str> = Some("Success");
            let user_info = "user123";

            output.insert("module", name);
            output.insert("fn", function_name);
            output.insert("timestamp", timestamp);
            output.insert("address", address);
            output.insert("args", args);
            output.insert("kwargs", kwargs);
            output.insert("result", result.unwrap_or(""));
            output.insert("user", user_info);

            output.entry("data").and_modify(|e| {
                let data = e.clone();
                e.clear();
                e.extend(data);
            });
            
            let latency = c.time() - timestamp.parse::<u64>().unwrap_or(0);
            output.insert("latency", &latency.to_string());
            let add_history: PyObject = obj.getattr('add_history');
            add_history.call1(output)
        }
        sender.send(result).await.expect("Failed to send result");
    })
}
#[tokio::main]
fn forward(py: Python, obj: &PyAny, fn_name: &str, input: &PyDict) -> PyResult<PyObject> {
    let (tx, rx) = mpsc::channel(1); // Channel for sending result
    
    // Spawn a new thread for each request
    tokio::spawn(async move {
        forward_worker(obj, fn_name, input, tx).await;
    });

    // Receive and return the result
    rx.recv().await.expect("Failed to receive result")
}

#[pymodule]
fn My_module(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(forward, m)?)?;
    Ok(())
}
