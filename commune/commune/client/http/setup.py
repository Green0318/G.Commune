from setuptools import setup
from setuptools_rust import RustExtension

setup(
    name="rust_http_module",
    version="0.1",
    rust_extensions=[RustExtension("rust_http_module.rust_http_module")],
    packages=["rust_http_module"],
    zip_safe=False,
)
