use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use std::collections::HashMap;
use std::time::Instant;


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




#[pyfunction]
fn worker2logs(_py: Python, _self: &PyAny, worker: &str) -> PyResult<HashMap<String, Vec<String>>> {
    let workers: Vec<String> = _self.call_method0("workers")?.extract()?;
    let mut worker2logs = HashMap::new();
    for w in workers {
        worker2logs.insert(w.clone(), _self.call_method1("logs", (w,))?.extract()?);
    }
    Ok(worker2logs)
}

#[getter]
fn worker_name_prefix(_py: Python, _self: &PyAny) -> PyResult<String> {
    Ok(format!("{}/{}", _self.getattr("server_name")?.extract::<String>()?, _self.getattr("worker_fn")?.extract::<String>()?))
}

#[pyfunction]
fn worker(_py: Python, _cls: &PyAny, args: Option<&PyTuple>, kwargs: Option<&PyDict>) -> PyResult<()> {
    let kwargs = kwargs.unwrap_or_else(|| PyDict::new(_py));
    kwargs.set_item("start", false)?;
    let self_ = _cls.call(*args, Some(kwargs))?;
    _py.run(|py| {
        let c = PyModule::import(py, "c")?;
        c.call1(("new_event_loop",), &[("nest_asyncio", true)])?;
        c.call1(("print",), &[("Running -> network: {} netuid: {}", (_self.getattr("config")?.getattr("network")?, _self.getattr("config")?.getattr("netuid")?))])?;
        
        let mut running = true;
        let mut last_print = Instant::now();
        let executor = c.call_method1("module", ("executor.thread",))?.call1(("max_workers", _self.getattr("config")?.getattr("num_threads")?))?;

        while running {
            let mut results = Vec::new();
            let mut futures = Vec::new();
            if _self.getattr("last_sync_time")?.extract::<Instant>()? + _self.getattr("config")?.getattr("sync_interval")?.extract::<Instant>()? < Instant::now() {
                c.call1(("print",), &[("Syncing network {}", _self.getattr("config")?.getattr("network")?), ("cyan",)])?;
                _self.call_method0("sync")?;
            }
            let module_addresses = c.call_method0("shuffle", (_self.call_method0("copy", (_self.getattr("module_addresses")?,))?,))?;
            let batch_size = _self.getattr("config")?.getattr("batch_size")?;
            for (i, module_address) in module_addresses.extract::<HashSet<String>>()?.into_iter().enumerate() {
                if futures.len() < batch_size.extract::<usize>()? {
                    let future = executor.call_method1("submit", (_self.call_method1("eval_module", (module_address,))?,))?;
                    futures.push(future);
                } else {
                    for ready_future in c.call_method1("as_completed", (futures,))? {
                        let ready_future = ready_future?;
                        let result = ready_future.call_method0("result")?;
                        futures.remove(ready_future)?;
                        results.push(result)?;
                        break;
                    }
                }
                if last_print.elapsed() > _self.getattr("config")?.getattr("print_interval")?.extract::<Instant>()? {
                    let stats = pyo3::types::PyDict::new(py);
                    stats.set_item("lifetime", _self.getattr("lifetime")?)?;
                    stats.set_item("pending", futures.len())?;
                    stats.set_item("sent", _self.getattr("requests")?)?;
                    stats.set_item("errors", _self.getattr("errors")?)?;
                    stats.set_item("successes", _self.getattr("successes")?)?;
                    c.call1(("print",), &[("{}", stats), ("cyan",)])?;
                    last_print = Instant::now();
                }
            }
        }
        Ok(())
    })
}

#[pyfunction]
fn sync(_py: Python, _self: &PyAny, network: Option<&str>, search: Option<&str>, netuid: Option<i32>, update: Option<bool>) -> PyResult<HashMap<String, PyObject>> {
    let network = network.unwrap_or_else(|| _self.getattr("config")?.getattr("network")?.extract::<String>().unwrap());
    let search = search.unwrap_or_else(|| _self.getattr("config")?.getattr("search")?.extract::<String>().unwrap());
    let netuid = netuid.unwrap_or_else(|| _self.getattr("config")?.getattr("netuid")?.extract::<i32>().unwrap());
    let update = update.unwrap_or(false);
    
    if network.contains("subspace") {
        let mut splits = network.split('.');
        let network = splits.next().unwrap_or_default();
        let netuid = splits.next().unwrap_or_default().parse::<i32>().unwrap_or_default();
        let subspace = _py.import("subspace")?;
        _self.setattr("subspace", subspace)?;
    } else {
        _self.delattr("subspace")?;
        _self.setattr("name2key", PyDict::new(_py))?;
    }

    _self.setattr("network", network)?;
    _self.setattr("netuid", netuid)?;
    
    let namespace = _self.call_method1("namespace", (search,))?;
    let n = namespace.getattr("__len__")?.call0()?.extract::<i32>()?;
    let module_addresses = namespace.call_method0("values")?.to_list(_py)?;
    let names = namespace.call_method0("keys")?.to_list(_py)?;
    let address2name = _py.eval("dict(zip(module_addresses, names))", None, None)?;
    _self.setattr("last_sync_time", Instant::now())?;

    let mut r = HashMap::new();
    r.insert("network".to_string(), network.to_object(_py));
    r.insert("netuid".to_string(), netuid.to_object(_py));
    r.insert("n".to_string(), n.to_object(_py));
    r.insert("timestamp".to_string(), Instant::now().to_object(_py));
    r.insert("msg".to_string(), "Synced network".to_object(_py));

    Ok(r)
}

#[pymodule]
fn mymodule(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run_loop, m)?)?;
    m.add_function(wrap_pyfunction!(run_info, m)?)?;
    m.add_function(wrap_pyfunction!(workers, m)?)?;
    m.add_function(wrap_pyfunction!(start_workers, m)?)?;
    m.add_function(wrap_pyfunction!(worker2logs, m)?)?;
    m.add_getter(wrap_pyfunction!(worker_name_prefix, m)?)?;
    m.add_function(wrap_pyfunction!(worker, m)?)?;
    m.add_function(wrap_pyfunction!(sync, m)?)?;
    Ok(())
}