use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use std::collections::HashMap;
extern crate serde_json;
use std::fs;
use std::path::Path;
use serde_json::Value;

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
    Ok(())
}

