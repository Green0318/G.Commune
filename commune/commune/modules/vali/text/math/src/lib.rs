use pyo3::prelude::*;

#[pyfunction]
fn score_module(obj: &PyAny, module: &PyAny, kwargs: Option<HashMap<&str, &str>>) -> PyResult<HashMap<String, PyObject>> {
    let mut w = 0.0;

    let dataset = obj.call_method0("dataset")?;
    let sample = dataset.call_method0("sample")?;
    let t = obj.call_method0("time")?;

    let prompt = format!(
        "INPUT (JSON):\n```\n{}\n```\nQUESTION:\n\nWAS THE INPUT REAL (1) OR TAMPERED (0)? -> :\n\nOUTPUT (answer: int):",
        sample
    );

    let output_text = module.call_method1("forward", (("generate",), [("args", prompt)]))?;
    let output_text_str: String = output_text.extract()?;
    let prediction = output_text_str.split("json```").nth(1).unwrap().split("```").next().unwrap().parse::<f64>().unwrap();
    let target = sample.get_item("answer")?.extract::<f64>()?;

    w = (prediction - target).abs();

    let mut response = HashMap::new();
    response.insert("prompt".to_string(), prompt.into_py_any(Python::acquire_gil()));
    response.insert("latency".to_string(), (obj.call_method0("time")? - t).into_py_any(Python::acquire_gil()));
    response.insert("target".to_string(), sample.get_item("real")?);
    response.insert("prediction".to_string(), prediction.into_py_any(Python::acquire_gil()));
    response.insert("output_text".to_string(), output_text.into_py_any(Python::acquire_gil()));
    response.insert("w".to_string(), w.into_py_any(Python::acquire_gil()));

    Ok(response)
}

#[pymodule]
fn my_module_vali_test_math(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(score_module, m)?)?;
    Ok(())
}
