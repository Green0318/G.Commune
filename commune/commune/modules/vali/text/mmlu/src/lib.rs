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
    let prompt = json!({
        "sample": dataset,
        "instruction": "GIVE THE ANSWER AS AN INDEX -> {{answer:str}} ?"
    }).to_string();

    let output: String = model.call_method1("generate", (prompt,))?.extract()?;
    
    let output_answer: String = if let Ok(output_dict) = output.parse::<HashMap<String, String>>() {
        output_dict.get("answer").unwrap_or("").to_string()
    } else {
        match output.parse::<HashMap<String, String>>() {
            Ok(output_map) => output_map.get("answer").unwrap_or("").to_string(),
            Err(_) => output,
        }
    };

    let w = if output_answer.contains(&target.to_string()) { 1.0 } else { 0.0 };

    let mut response = HashMap::new();
    response.insert("w".to_string(), w.into_py_any(Python::acquire_gil()));
    response.insert("target".to_string(), target.into_py_any(Python::acquire_gil()));
    response.insert("input".to_string(), prompt.into_py_any(Python::acquire_gil()));
    response.insert("output".to_string(), output_answer.into_py_any(Python::acquire_gil()));

    Ok(response)
}

#[pymodule]
fn my_module_text_mmlu(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(score_module, m)?)?;
    Ok(())
}
