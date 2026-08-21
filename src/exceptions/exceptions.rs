use pyo3::create_exception;
use pyo3::exceptions::PyException;

create_exception!(tryx, FailedBuildClient, PyException);
create_exception!(tryx, FailedToDecodeProto, PyException);
create_exception!(tryx, UnsupportedEventType, PyException);
create_exception!(tryx, UnsupportedBackend, PyException);
create_exception!(tryx, EventDispatchError, PyException);
create_exception!(tryx, PyPayloadBuildError, PyException);
