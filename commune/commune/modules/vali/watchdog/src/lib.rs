use pyo3::prelude::*;

#[pyclass]
struct Demo {
    config: HashMap<String, PyObject>,
}

#[pymethods]
impl Demo {
    #[new]
    fn new(a: i32, b: i32) -> Self {
        let mut config = HashMap::new();
        config.insert("a".to_string(), a.into_py_any(Python::acquire_gil()));
        config.insert("b".to_string(), b.into_py_any(Python::acquire_gil()));
        Demo { config }
    }

    fn call(&self, x: i32, y: i32) -> PyResult<i32> {
        Python::with_gil(|py| {
            println!("{:?}", self.config);
            println!("{:?}", self.config); // Assuming print is overridden in Python
            Ok(x + y)
        })
    }
}

#[pymodule]
fn my_module_watchdog(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Demo>()?;
    Ok(())
}
