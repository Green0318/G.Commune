use pyo3::prelude::*;

#[pyfunction]
fn score_module(obj: &PyAny, module: PyObject, kwargs: Option<HashMap<&str, &str>>) -> PyResult<HashMap<String, PyObject>> {
    let module = if let Ok(module_str) = module.extract::<String>() {
        obj.call_method1("connect", (module_str,))?
    } else {
        module
    };

    let dataset = obj.getattr("dataset")?.call_method0("sample", ())?;
    let target = dataset.get_item(obj.getattr("config")?.getattr("target")?)?;
    let prompt = obj.call_method1("create_prompt", (dataset,))?;
    let output = module.call_method1("generate", (prompt,))?;
    let prediction = output.to_string();
    let w = if prediction.contains(&target.to_string()) { 1 } else { 0 };

    let mut response = HashMap::new();
    response.insert("sample".to_string(), dataset.into_py_any(Python::acquire_gil()));
    response.insert("target".to_string(), target.into_py_any(Python::acquire_gil()));
    response.insert("prediction".to_string(), prediction.into_py_any(Python::acquire_gil()));
    response.insert("output".to_string(), output.into_py_any(Python::acquire_gil()));
    response.insert("w".to_string(), w.into_py_any(Python::acquire_gil()));

    Ok(response)
}

#[pymodule]
fn my_module_vali_text_realfake(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(score_module, m)?)?;
    Ok(())
}
