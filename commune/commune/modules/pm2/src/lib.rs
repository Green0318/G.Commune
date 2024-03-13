use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use std::collections::HashMap;
extern crate serde_json;
use std::fs;
use std::path::Path;
use serde_json::Value;
use std::collections::HashMap;
use pyo3::wrap_pyfunction;


#[pyfunction]
fn restart(py: Python, name: &str, verbose: bool, prefix_match: bool) -> PyResult<HashMap<String, PyObject>> {
    let c = py.import("c")?;
    let list = list(py, c)?;

    let mut rm_list = Vec::new();
    if list.contains(&name.to_string()) {
        rm_list.push(name);
    } else {
        if prefix_match {
            for p in &list {
                if p.starts_with(name) {
                    rm_list.push(p);
                }
            }
        } else {
            return Err(PyValueError::new_err(format!("pm2 process {} not found", name)));
        }
    }

    if rm_list.is_empty() {
        return Ok(HashMap::new());
    }

    let mut restart_results = HashMap::new();
    for n in rm_list {
        if verbose {
            let print = c.getattr("print")?;
            print.call1((format!("Restarting {}", n), "cyan"))?;
        }
        let cmd = c.getattr("cmd")?;
        cmd.call1((format!("pm2 restart {}", n), false))?;
        let cls = c.getattr("cls")?;
        rm_logs(py, cls, n)?;
        restart_results.insert("success".to_string(), true.into_py(py));
        restart_results.insert("message".to_string(), format!("Restarted {}", name).into_py(py));
    }

    Ok(restart_results)
}

#[pyfunction]
fn restart_prefix(py: Python, cls: &PyAny, name: Option<&str>, verbose: bool) -> PyResult<Vec<String>> {
    let c = py.import("c")?;
    let list = list(py, cls)?;
    let mut restarted_modules = Vec::new();

    for module in list {
        if module.starts_with(name.unwrap_or_default()) || name.unwrap_or_default() == "all" {
            if verbose {
                let print = c.getattr("print")?;
                print.call1((format!("Restarting {}", module), "cyan"))?;
            }
            let cmd = c.getattr("cmd")?;
            cmd.call1((format!("pm2 restart {}", module), verbose))?;
            restarted_modules.push(module);
        }
    }

    Ok(restarted_modules)
}

#[pyfunction]
fn kill_many(py: Python, cls: &PyAny, search: Option<&str>, verbose: bool, timeout: i32) -> PyResult<Vec<PyObject>> {
    let c = py.import("c")?;
    let list = list(py, cls, search)?;
    let mut futures = Vec::new();

    for name in list {
        let print = c.getattr("print")?;
        print.call1((format!("[bold cyan]Killing[/bold cyan] [bold yellow]{}[/bold yellow]", name), "green"))?;

        let submit = c.getattr("submit")?;
        let kwargs = [("name", name), ("verbose", verbose)];
        let kwargs_dict = PyDict::new(py);
        for &(key, value) in kwargs.iter() {
            kwargs_dict.set_item(key, value)?;
        }
        let f: PyObject = submit.call1((cls, kwargs_dict, true, timeout))?.extract()?;
        futures.push(f);
    }

    let wait = c.getattr("wait")?;
    wait.call1((futures,))
}

#[pyfunction]
fn kill_all(py: Python, cls: &PyAny, verbose: bool, timeout: i32) -> PyResult<Vec<PyObject>> {
    kill_many(py, cls, None, verbose, timeout)
}


fn list(py: Python, c: &PyModule) -> PyResult<Vec<String>> {
    let cmd = c.getattr("cmd")?;
    let output_string: String = cmd.call1(("pm2 status", false))?.extract()?;
    let mut module_list = Vec::new();
    for line in output_string.lines() {
        if line.contains("  default  ") {
            let server_name = line.split("default").next().unwrap().trim().split_whitespace().last().unwrap().to_string();
            module_list.push(server_name);
        }
    }
    Ok(module_list)
} 

fn rm_logs(py: Python, cls: &PyAny, name: String) -> PyResult<()> {
    let logs_map = logs_path_map(py, cls, name)?;
    let c = py.import("c")?;
    for k in logs_map.keys() {
        let rm = c.getattr("rm")?;
        rm.call1((logs_map[k].clone(),))?;
    }
    Ok(())
}

#[pyfunction]
fn exists(cls: &PyAny, name: &str) -> PyResult<bool> {
    let list = cls.call_method0("list")?;
    let list: Vec<String> = list.extract()?;
    Ok(list.contains(&name.to_string()))
}

#[pyfunction]
fn start(
    cls: &PyAny,
    path: &str,
    name: &str,
    cmd_kwargs: Option<&str>,
    refresh: bool,
    verbose: bool,
    force: bool,
    interpreter: Option<&str>,
) -> PyResult<HashMap<String, PyObject>> {
    let exists = exists(cls, name)?;
    if exists && refresh {
        cls.call_method1("kill", (name, verbose),)?;
    }

    let mut cmd = format!("pm2 start {} --name {}", path, name);
    if force {
        cmd += " -f";
    }

    if let Some(interpreter) = interpreter {
        cmd += &format!(" --interpreter {}", interpreter);
    }

    if let Some(cmd_kwargs) = cmd_kwargs {
        cmd += " -- ";
        if let Ok(cmd_kwargs_dict) = serde_json::from_str::<HashMap<String, String>>(cmd_kwargs) {
            for (k, v) in cmd_kwargs_dict.iter() {
                cmd += &format!("--{} {}", k, v);
            }
        } else {
            cmd += cmd_kwargs;
        }
    }

    let print = cls.getattr("print")?;
    print.call1((format!("[bold cyan]Starting (PM2)[/bold cyan] [bold yellow]{}[/bold yellow]", name), "green"))?;

    let cmd_result = cls.call_method1("cmd", (cmd, verbose))?;

    let mut result = HashMap::new();
    result.insert("success".to_string(), true.to_object(py));
    result.insert("message".to_string(), format!("Launched {}", name).to_object(py));
    result.insert("command".to_string(), cmd.to_object(py));
    result.insert("stdout".to_string(), cmd_result);
    Ok(result)
}

#[pyfunction]
fn launch(
    module: Option<&str>,
    fn_name: &str,
    name: Option<&str>,
    tag: Option<&str>,
    args: Option<Vec<&str>>,
    kwargs: Option<HashMap<&str, &str>>,
    device: Option<&str>,
    interpreter: &str,
    auto: bool,
    verbose: bool,
    force: bool,
    meta_fn: &str,
    tag_separator: &str,
    cwd: Option<&str>,
    refresh: bool,
    py: Python,
    cls: &PyAny,
) -> PyResult<HashMap<String, PyObject>> {
    let module_path = if let Some(module) = module {
        module
    } else {
        let module = cls.call_method0("module_path")?;
        module.extract::<String>()?
    };

    let args = args.unwrap_or_default();
    let kwargs = kwargs.unwrap_or_default();

    let kwargs_json = json!({
        "module": module_path,
        "fn": fn_name,
        "args": args,
        "kwargs": kwargs,
    });

    let kwargs_str = serde_json::to_string(&kwargs_json)?;
    let name = cls.call_method1("resolve_server_name", (module_path, name, tag, tag_separator))?.extract::<String>()?;

    if refresh {
        cls.call_method1("kill", (name.clone(),))?;
    }

    let module = cls.call_method0("module")?;
    let file_path = module.call_method0("filepath")?.extract::<String>()?;

    let mut command = format!("pm2 start {} --name {} --interpreter {}", file_path, name, interpreter);
    if !auto {
        command += " --no-autorestart";
    }
    if force {
        command += " -f";
    }
    command += &format!(" -- --fn {} --kwargs '{}'", meta_fn, kwargs_str);

    let mut env = HashMap::new();
    if let Some(device) = device {
        if device.parse::<i32>().is_ok() {
            env.insert("CUDA_VISIBLE_DEVICES".to_string(), device.to_string());
        } else {
            let device_list: Vec<String> = device.split(',').map(|s| s.to_string()).collect();
            env.insert("CUDA_VISIBLE_DEVICES".to_string(), device_list.join(","));
        }
    }

    if refresh {
        cls.call_method1("kill", (name.clone(),))?;
    }

    let cwd = cwd.unwrap_or_else(|| module.call_method0("dirpath").unwrap().extract::<String>().unwrap());
    let cmd_result = cls.call_method1("cmd", (command.clone(),)).unwrap();

    let mut result = HashMap::new();
    result.insert("success".to_string(), true.to_object(py));
    result.insert("message".to_string(), format!("Launched {}", module_path).to_object(py));
    result.insert("command".to_string(), command.to_object(py));
    result.insert("stdout".to_string(), cmd_result);
    Ok(result)
}

fn logs_path_map(py: Python, cls: &PyAny, name: Option<&str>) -> PyResult<HashMap<String, HashMap<String, String>>> {
    let c = py.import("c")?;
    let logs_path_map = HashMap::new();
    let logs = c.getattr("ls")?;
    let logs_output: String = logs.call1((format!("{}/logs/", cls.getattr("dir")?.extract::<String>()?),))?.extract()?;
    let mut logs_path_map = HashMap::new();

    for l in logs_output.lines() {
        let key_parts: Vec<&str> = l.split('/').collect();
        let key = key_parts[key_parts.len() - 1].split('-').take_while(|x| x != &"").collect::<Vec<&str>>().join(":");
        let entry = logs_path_map.entry(key).or_insert(Vec::new());
        entry.push(l);
    }

    for (_, value) in logs_path_map.iter_mut() {
        let mut map = HashMap::new();
        for l in value.iter() {
            let parts: Vec<&str> = l.split('-').collect();
            let file_name_parts: Vec<&str> = parts[parts.len() - 1].split('.').collect();
            map.insert(file_name_parts[0].to_string(), l.to_string());
        }
        *value = map;
    }

    match name {
        Some(name) => {
            match logs_path_map.get(name) {
                Some(result) => Ok(result.clone()),
                None => Ok(HashMap::new()),
            }
        }
        None => Ok(logs_path_map),
    }
}

#[pymodule]
fn mymodule(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(restart, m)?)?;
    m.add_function(wrap_pyfunction!(restart_prefix, m)?)?;
    m.add_function(wrap_pyfunction!(kill_many, m)?)?;
    m.add_function(wrap_pyfunction!(kill_all, m)?)?;
    m.add_function(wrap_pyfunction!(exists, m)?)?;
    m.add_function(wrap_pyfunction!(start, m)?)?;
    m.add_function(wrap_pyfunction!(launch, m)?)?;
    Ok(())
}









