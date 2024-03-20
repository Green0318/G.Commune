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


#[pyclass]
struct NetworkConfig {
    network: String,
    search: String,
    netuid: i32,
}

#[pymethods]
impl NetworkConfig {
    #[new]
    fn new() -> Self {
        NetworkConfig {
            network: String::new(),
            search: String::new(),
            netuid: 0,
        }
    }
}

#[pyfunction]
fn set_network(obj: &PyAny, network: Option<&str>, search: Option<&str>, netuid: Option<i32>, update: bool) -> PyResult<HashMap<&str, Py<PyAny>>> {
    let network_config: Py<NetworkConfig> = obj.extract()?;
    let mut network_config = network_config.borrow_mut();

    let network = match network {
        Some(n) => n.to_string(),
        None => network_config.network.clone(),
    };

    let search = match search {
        Some(s) => s.to_string(),
        None => network_config.search.clone(),
    };

    let netuid = match netuid {
        Some(n) => n,
        None => network_config.netuid,
    };

    // Handling for "subspace" network
    if network.contains("subspace") {
        if network.contains(".") {
            let splits: Vec<&str> = network.split('.').collect();
            assert_eq!(splits.len(), 2, "Network must be in the form of {{network}}.{{subnet/netuid}}, got {}", network_config.network);
            let netuid_str = splits[1];
            let netuid = netuid_str.parse::<i32>().unwrap();
            network_config.network = splits[0].to_string();
            network_config.netuid = netuid;
        } else {
            network_config.network = "subspace".to_string();
            network_config.netuid = 0;
        }
        // Assuming `c.module` and `c.namespace` functions are already implemented
        // Adjust this part accordingly
        // self.subspace = c.module("subspace")(netuid=netuid);
    } else {
        // Assuming `name2key` is a field of `NetworkConfig`
        network_config.name2key.clear();
    }

    // Assuming `c.namespace` and `c.time` functions are already implemented
    // Adjust this part accordingly
    // self.namespace = c.namespace(search=search, network=network, netuid=netuid, update=update);
    let namespace = HashMap::new(); // Placeholder
    let n = namespace.len();
    let address2name: HashMap<&str, &str> = namespace.iter().map(|(k, v)| (v, k)).collect();
    let last_sync_time = 0; // Placeholder

    network_config.network = network.clone();
    network_config.search = search.clone();
    network_config.netuid = netuid;

    let mut result = HashMap::new();
    result.insert("network", network.to_object(obj.py()));
    result.insert("netuid", netuid.to_object(obj.py()));
    result.insert("n", n.to_object(obj.py()));
    result.insert("timestamp", last_sync_time.to_object(obj.py()));
    result.insert("msg", "Synced network".to_object(obj.py()));

    Ok(result)
}



#[pyfunction]
fn eval_module(obj: &PyAny, module: &str) -> PyResult<HashMap<&str, Py<PyAny>>> {
    // Load module stats (if exists)

    // Load module info and calculate staleness
    // If the module is stale, return the module info
    let info = obj.call_method1("get_module_info", (module,))?;
    let module = obj.call_method1("connect", (info.get_item("address").unwrap(),), Some("key".into()))?;
    let requests: usize = obj.getattr("requests")?.extract()?;
    let config_max_staleness: f64 = obj.getattr("config").unwrap().getattr("max_staleness")?.extract()?;
    let time_now: f64 = obj.call_method0("time")?.extract()?;
    let seconds_since_called = time_now - info.get_item("timestamp").unwrap().extract::<f64>()?;
    if seconds_since_called < config_max_staleness {
        return Ok(hashmap!{
            "w" => info.get("w").unwrap_or(0),
            "module" => info["name"],
            "address" => info["address"],
            "timestamp" => time_now,
            "msg" => format!("Module is not stale, {} < {}", seconds_since_called, config_max_staleness)
        });
    }

    let module_info = module.call_method0("info")?;
    assert!(info.contains("address") && info.contains("name"));

    // Make sure module info has a timestamp
    let mut info_dict: HashMap<&str, Py<PyAny>> = info.extract()?;
    info_dict.extend(module_info.extract::<HashMap<&str, Py<PyAny>>>()?);
    info_dict.insert("timestamp", time_now);

    let response = obj.call_method1("score_module", (module,))?;
    let response_checked = obj.call_method1("check_response", (response,))?;

    let successes: usize = obj.getattr("successes")?.extract()?;
    let alpha: f64 = obj.getattr("config").unwrap().getattr("alpha")?.extract()?;
    let w: f64 = response_checked["w"].extract()?;
    let latency = time_now - info_dict["timestamp"].extract::<f64>()?;
    let path = format!("{}/{}", obj.getattr("storage_path")?.extract::<String>()?, info_dict["name"].extract::<String>()?);
    obj.call_method1("put_json", (path.clone(), info_dict))?;

    let mut result = hashmap!{
        "w" => w * alpha + info_dict["w"].extract::<f64>()? * (1.0 - alpha),
        "module" => info_dict["name"],
        "address" => info_dict["address"],
        "latency" => latency
    };

    if let Ok(emoji_check) = obj.call_method0("emoji", ("checkmark",)) {
        result.insert("msg", format!("{}{} --> w:{} {}", emoji_check, info_dict["name"], w, emoji_check));
    } else {
        result.insert("msg", format!("{} {} {}", obj.call_method0("emoji", ("cross",))?, info_dict["name"], obj.call_method0("emoji", ("cross",))?));
    }

    Ok(result)
}


#[pyfunction]
fn vote(obj: &PyAny, async_vote: bool, save: bool, kwargs: Option<HashMap<&str, &str>>) -> PyResult<HashMap<&str, Py<PyAny>>> {
    if async_vote {
        return obj.call_method1("submit", (obj.getattr("vote")?,));
    }

    let votes = obj.call_method0("votes")?;
    let config_min_num_weights: usize = obj.getattr("config")?.getattr("min_num_weights")?.extract()?;
    let votes_uids_len: usize = votes.get_item("uids").unwrap().extract()?;
    if votes_uids_len < config_min_num_weights {
        return Ok(hashmap!{
            "success" => false,
            "msg" => "The votes are too low",
            "votes" => votes_uids_len,
            "min_num_weights" => config_min_num_weights
        });
    }

    let r = obj.call_method1("vote", (votes.get_item("uids").unwrap(), votes.get_item("weights").unwrap(), obj.getattr("key")?, obj.getattr("config")?.getattr("network")?, obj.getattr("config")?.getattr("netuid")?))?;
    
    if save {
        obj.call_method1("save_votes", (votes,))?;
    }

    let time_now: f64 = obj.call_method0("time")?.extract()?;
    obj.setattr("last_vote_time", time_now)?;

    Ok(hashmap!{
        "success" => true,
        "message" => "Voted",
        "num_uids" => votes_uids_len,
        "timestamp" => time_now,
        "avg_weight" => obj.call_method1("mean", (votes.get_item("weights").unwrap(),))?,
        "stdev_weight" => obj.call_method1("stdev", (votes.get_item("weights").unwrap(),))?,
        "r" => r
    })
}

#[pyfunction]
fn dashboard(obj: &PyAny) -> PyResult<()> {
    let streamlit = Python::acquire_gil().python().import("streamlit")?;
    let module_path: String = obj.call_method0("path")?.extract()?;
    obj.call_method0("load_style")?;
    let new_event_loop = obj.call_method0("new_event_loop")?;
    let title = streamlit.call_method1("title", (module_path.clone(),))?;
    let servers = obj.call_method0("servers")?;
    let server = streamlit.call_method1("selectbox", ("Select Vali", servers))?;
    let state_path = format!("dashboard/{}", server);
    let module = obj.call_method1("module", (server.clone(),))?;
    let state = module.call_method1("get", (state_path.clone(), {}))?;
    let server = obj.call_method1("connect", (server.clone(),))?;

    let module_infos = if state.len() == 0 {
        let run_info = server.getattr("run_info")?;
        let module_infos = server.call_method0("module_infos")?;
        let state = hashmap!{
            "run_info" => run_info,
            "module_infos" => module_infos
        };
        obj.call_method1("put", (state_path.clone(), state))?;
        module_infos
    } else {
        state.get_item("module_infos")?
    };

    let mut df = vec![];
    let selected_columns = ["name", "address", "w", "staleness"];
    let selected_columns = streamlit.call_method1("multiselect", ("Select columns", selected_columns, selected_columns))?;
    let search = streamlit.call_method0("text_input", ("Search",))?;
    for row in module_infos.iter::<PyList>()? {
        let row = row?;
        if !search.is_none() && !row.get_item("name").unwrap().extract::<String>()?.contains(&search.extract::<String>()?) {
            continue;
        }
        let mut row_dict = HashMap::new();
        for column in selected_columns.iter::<PyList>()? {
            let column = column?;
            row_dict.insert(column.extract::<String>()?, row.get_item(column)?);
        }
        df.push(row_dict);
    }
    if df.len() == 0 {
        streamlit.call_method1("write", ("No modules found",))?;
    } else {
        let default_columns = ["w", "staleness"];
        let mut sorted_columns = vec![];
        for c in &default_columns {
            if df.get_item(c).unwrap() {
                sorted_columns.push(c.clone());
            }
        }
        df.sort_by(|a, b| {
            let mut comparison = 0;
            for c in sorted_columns.iter() {
                if a.get(c) > b.get(c) {
                    comparison = -1;
                    break;
                } else if a.get(c) < b.get(c) {
                    comparison = 1;
                    break;
                }
            }
            comparison
        });
        streamlit.call_method1("write", (df,))?;
    }
    Ok(())
}

#[pyfunction]
fn vote_loop(obj: &PyAny) -> PyResult<()> {
    loop {
        if obj.getattr("should_vote")?.extract::<bool>()? {
            let futures = vec![obj.call_method0("submit", (obj.getattr("vote")?,))?];
            for ready_future in futures {
                let result = ready_future?.call_method0("result")?;
                obj.call_method1("print", (result,))?;
            }
        }
        obj.call_method1("print", (obj.call_method0("run_info")?.result()?,))?;
        obj.call_method1("sleep", (obj.getattr("config")?.getattr("sleep_interval")?,))?;
    }
}



#[pymodule]
fn mymodule_vali(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run_loop, m)?)?;
    m.add_function(wrap_pyfunction!(run_info, m)?)?;
    m.add_function(wrap_pyfunction!(workers, m)?)?;
    m.add_function(wrap_pyfunction!(start_workers, m)?)?;
    m.add_function(wrap_pyfunction!(worker2logs, m)?)?;
    m.add_getter(wrap_pyfunction!(worker_name_prefix, m)?)?;
    m.add_function(wrap_pyfunction!(worker, m)?)?;
    m.add_function(wrap_pyfunction!(sync, m)?)?;
    m.add_class::<NetworkConfig>()?;
    m.add_function(wrap_pyfunction!(set_network, m)?)?;
    m.add_function(wrap_pyfunction!(eval_module, m)?)?;
    m.add_function(wrap_pyfunction!(vote, m)?)?;
    m.add_function(wrap_pyfunction!(dashboard, m)?)?;
    m.add_function(wrap_pyfunction!(vote_loop, m)?)?;

    Ok(())
}