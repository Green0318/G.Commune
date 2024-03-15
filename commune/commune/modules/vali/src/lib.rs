use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use std::collections::HashMap;


#[pyfunction]
fn run_loop(_py: Python, _self: &PyAny) -> PyResult<()> {
    let c = PyModule::import(_self.get_type().module()?, _py)?;

    c.call1(("Vali config: {:?}", _self.getattr("config")?), "cyan")?;

    let config = _self.getattr("config")?;
    let vote_tag = config.getattr("vote_tag")?.extract::<Option<PyObject>>()?;

    if vote_tag.is_none() {
        _self.call_method1("start_workers", (_self.getattr("num_workers")?, _self.getattr("refresh")?, _self.getattr("mode")?))?;
        let mut steps = 0;
        c.call1(("Vali loop started",), "cyan")?;
        loop {
            steps += 1;
            c.call1((format!("Vali loop step {}", steps),), "cyan")?;
            let run_info = _self.call_method0("run_info")?;
            if let Ok(run_info) = run_info.extract::<HashMap<String, PyObject>>() {
                let network = config.getattr("network")?.extract::<String>()?;
                if network.contains("subspace") && run_info.get("vote_staleness").unwrap_or(&0).extract::<i32>()? > config.getattr("vote_interval")?.extract::<i32>()? {
                    let response = _self.call_method0("vote")?;
                    c.print(response)?;
                }
                c.print(run_info)?;
                c.call1(("Sleeping... for {}", config.getattr("run_loop_sleep")?), "cyan")?;
                c.call_method1("sleep", (config.getattr("run_loop_sleep")?,))?;
            }
        }
    }
    Ok(())
}

#[pyfunction]
fn run_info(_py: Python, _self: &PyAny) -> PyResult<HashMap<String, PyObject>> {
    let mut info = HashMap::new();
    info.insert("lifetime".to_string(), _self.getattr("lifetime")?);
    info.insert("vote_staleness".to_string(), _self.getattr("vote_staleness")?);
    info.insert("errors".to_string(), _self.getattr("errors")?);
    let config = _self.getattr("config")?;
    info.insert("vote_interval".to_string(), config.getattr("vote_interval")?);
    info.insert("epochs".to_string(), _self.getattr("epochs")?);
    info.insert("workers".to_string(), _self.call_method0("workers")?);
    Ok(info)
}

#[pyfunction]
fn workers(_py: Python, _self: &PyAny) -> PyResult<Vec<String>> {
    let c = PyModule::import(_self.get_type().module()?, _py)?;
    Ok(c.call_method0("pm2ls")?.extract()?)
}

#[pyfunction]
fn start_workers(_py: Python, _self: &PyAny, num_workers: i32, refresh: bool, mode: &str) -> PyResult<Vec<PyObject>> {
    let mut responses = Vec::new();
    let config = _self.call_method1("copy", (_self.getattr("config")?,))?;
    let config = _self.call_method1("munch2dict", (config,))?;
    if mode == "process" {
        for i in 0..num_workers {
            let name = format!("{}_{}", _self.getattr("worker_name_prefix")?.extract::<String>()?, i);
            if !refresh && _self.call_method1("pm2_exists", (name.clone(),))? {
                let msg = format!("Worker {} already exists, skipping", name);
                let color = "yellow";
                _self.call1((msg, color))?;
                continue;
            }
            let r = _self.call_method1("remote_fn", (("worker_fn",), ("name", name.clone()), ("refresh", refresh), ("kwargs", config.clone())))?;
            let msg = format!("Started worker {} {}", i, r);
            let color = "cyan";
            _self.call1((msg, color))?;
            responses.push(r);
        }
    } else if mode == "thread" {
        for i in 0..num_workers {
            let worker = _self.call_method1("thread", (("worker",), ("kwargs", config.clone())))?;
            let msg = format!("Started worker {} {}", i, worker);
            let color = "cyan";
            _self.call1((msg, color))?;
            responses.push(worker);
        }
    }
    Ok(responses)
}

#[pymodule]
fn mymodule(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run_loop, m)?)?;
    m.add_function(wrap_pyfunction!(run_info, m)?)?;
    m.add_function(wrap_pyfunction!(workers, m)?)?;
    m.add_function(wrap_pyfunction!(start_workers, m)?)?;
    Ok(())
}
