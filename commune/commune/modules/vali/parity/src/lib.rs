use pyo3::prelude::*;
use std::collections::HashMap;

#[pyfunction]
fn regloop(obj: &PyAny, n: usize, tag: &str, remote: bool, key: Option<&str>, timeout: usize) -> PyResult<()> {
    if remote {
        let mut kwargs = HashMap::new();
        kwargs.insert("n", n);
        kwargs.insert("tag", tag);
        kwargs.insert("remote", false);
        if let Some(k) = key {
            kwargs.insert("key", k);
        }
        kwargs.insert("timeout", timeout);
        return obj.call_method1("remote_fn", ("regloop", kwargs))?;
    }

    let mut cnt = 0;
    let cls = obj.call_method0("run", ())?;
    let subspace = obj.call_method0("module", ("subspace",))?.call_method0("namespace", (obj.call_method0("module_path", ())?,))?;
    let ip: String = obj.call_method0("ip", ())?.extract()?;
    let mut i = 0;
    let mut name2futures = HashMap::new();

    loop {
        let mut registered_servers = vec![];
        let namespace: HashMap<String, String> = subspace.call_method0("namespace", (obj.call_method0("module_path", ())?,))?.extract()?;
        obj.call_method1("print", ("registered servers", &namespace))?;
        while cnt < n {
            let name = obj.call_method1("resolve_server_name", ((tag.to_string() + &i.to_string()).as_str()))?;
            let module_key = obj.call_method1("get_key", (name.clone(),))?.get_item("ss58_address")?.extract::<String>()?;

            let mut futures = name2futures.entry(name.clone()).or_insert(vec![]);

            let address = format!("{}:{}", ip, 30333 + cnt);

            if namespace.contains_key(&name) {
                i += 1;
                obj.call_method1("print", ("already registered", &name))?;
                continue;
            }

            obj.call_method1("print", ("registering", &name))?;
            let response = obj.getattr("subspace")?.call_method1("register", (name.clone(), address.clone(), module_key, key.unwrap_or_default()))?;
            if !response.get_item("success")?.extract::<bool>()? {
                if response.get_item("error")?.get_item("name")?.extract::<String>()? == "NameAlreadyRegistered" {
                    i += 1;
                }
            }
            obj.call_method1("print", (response,))?;
            cnt += 1;
        }
    }
}
use pyo3::prelude::*;

#[pyfunction]
fn vote_loop(cls: &PyAny, remote: bool, min_staleness: usize, search: &str, namespace: Option<&PyAny>, network: Option<&PyAny>) -> PyResult<()> {
    if remote {
        let mut kwargs = HashMap::new();
        kwargs.insert("remote", false);
        if let Some(ns) = namespace {
            kwargs.insert("namespace", ns);
        }
        if let Some(net) = network {
            kwargs.insert("network", net);
        }
        return cls.call_method1("remote_fn", ("voteloop", kwargs))?;
    }

    let self_ = cls.call_method0("run", ())?;
    let stats = self_.getattr("stats")?.call_method1((search, false))?;
    for module in stats.iter::<PyList>()? {
        let module = module?;
        cls.call_method1("print", ("voting for", module.get_item("name")?.extract::<String>()?))?;
        if module.get_item("last_update")?.extract::<usize>()? > min_staleness {
            cls.call_method1("print", ("skipping", module.get_item("name")?.extract::<String>()?, "because it was voted recently"))?;
        }
        cls.call_method1("vote", (module.get_item("name")?.extract::<String>()?,))?;
    }
    Ok(())
}

#[pymodule]
fn my_module_vali_parity(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(regloop, m)?)?;
    m.add_function(wrap_pyfunction!(vote_loop, m)?)?;
    Ok(())
}
