use pyo3::prelude::*;

#[pyfunction]
fn score_module(obj: &PyAny, module: PyObject, kwargs: Option<HashMap<&str, &str>>) -> PyResult<HashMap<String, PyObject>> {
    let model = if let Ok(module_str) = module.extract::<String>() {
        obj.call_method1("connect", (module_str,))?
    } else {
        module
    };
    
    let dataset = obj.getattr("dataset")?.call_method0("sample", ())?;
    let target = dataset.get_item(obj.getattr("config")?.getattr("target")?)?;
    let prompt = obj.call_method1("create_prompt", (dataset,))?;
    let output: String = model.call_method1("generate", (prompt,))?.extract()?;
    println!("{}", output);
    let prediction = json::parse(&output)?.get("answer").unwrap_or("").to_string();

    let w = if target.contains(&prediction) { 1.0 } else { obj.getattr("config")?.getattr("base_score")?.extract()? };

    let mut response = HashMap::new();
    response.insert("w".to_string(), w.into_py_any(Python::acquire_gil()));
    response.insert("target".to_string(), target.into_py_any(Python::acquire_gil()));
    response.insert("prediction".to_string(), prediction.into_py_any(Python::acquire_gil()));
    response.insert("sample".to_string(), dataset.into_py_any(Python::acquire_gil()));
    response.insert("output".to_string(), output.into_py_any(Python::acquire_gil()));

    Ok(response)
}

#[pymodule]
fn my_module_vali_text_truthqa(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(score_module, m)?)?;
    Ok(())
}
