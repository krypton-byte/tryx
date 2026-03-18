use std::ops::Deref;

use pyo3::types::{PyList, PyString};
use pyo3::{Python, pyclass, pymethods, types::PyBytes};
use pyo3::{IntoPyObjectExt, prelude::*};
use whatsapp_rust::NodeBuilder;
use crate::types::JID;

pub enum NodeValueEnum {
    String(String),
    Jid(pyo3::Py<JID>),
}

/// PyClass wrapper untuk NodeValue
#[pyclass]
pub struct NodeValue {
    inner: NodeValueEnum,
}

#[pymethods]
impl NodeValue {
    #[new]
    pub fn new_string(value: String) -> Self {
        Self { inner: NodeValueEnum::String(value) }
    }

    #[staticmethod]
    pub fn jid(value: pyo3::Py<JID>) -> Self {
        Self { inner: NodeValueEnum::Jid(value) }
    }

    pub fn set_string(&mut self, value: String) {
        self.inner = NodeValueEnum::String(value);
    }

    pub fn set_jid(&mut self, value: pyo3::Py<JID>) {
        self.inner = NodeValueEnum::Jid(value);
    }
    #[getter]
    pub fn value(&self, py: Python<'_>) -> Py<PyAny> {
        match &self.inner {
            NodeValueEnum::String(s) => PyString::new(py, s).into_any().unbind(),
            NodeValueEnum::Jid(jid) => jid.clone_ref(py).into_any(),
        }
    }
    #[setter]
    pub fn set_value(&mut self, value: Py<PyAny>) {
        Python::attach(|py| {
            if let Ok(s) = value.extract::<String>(py) {
                self.inner = NodeValueEnum::String(s);
            } else if let Ok(jid) = value.extract::<pyo3::Py<JID>>(py) {
                self.inner = NodeValueEnum::Jid(jid);
            } else {
                panic!("Invalid type for NodeValue. Expected String or JID.");
            }
        });
    }

    pub fn __repr__(&self, py: Python<'_>) -> String {
        match &self.inner {
            NodeValueEnum::String(s) => format!("NodeValue::String({})", s),
            NodeValueEnum::Jid(jid) => {
                let jid_ref = jid.bind(py).borrow();
                format!("NodeValue::Jid({})", jid_ref.__repr__())
            }
        }
    }
}

/// NodeContent enum internal
pub enum NodeContentEnum {
    Bytes(Py<PyBytes>),
    String(Py<PyString>),
    Nodes(Py<PyList>), // PyList<Node>
}

/// PyClass wrapper untuk NodeContent
#[pyclass]
pub struct NodeContent {
    inner: NodeContentEnum,
}

#[pymethods]
impl NodeContent {
    #[getter]
    fn value(&self, py: Python<'_>) -> Py<PyAny> {
        match &self.inner {
            NodeContentEnum::Bytes(b) => b.clone_ref(py).into_any(),
            NodeContentEnum::String(s) => s.clone_ref(py).into_any(),
            NodeContentEnum::Nodes(n) => n.clone_ref(py).into_any(),
        }
    }

    pub fn is_bytes(&self) -> bool {
        matches!(self.inner, NodeContentEnum::Bytes(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self.inner, NodeContentEnum::String(_))
    }

    pub fn is_nodes(&self) -> bool {
        matches!(self.inner, NodeContentEnum::Nodes(_))
    }

    // pub fn __repr__(&self, py: Python<'_>) -> String {
    //     match &self.inner {
    //         NodeContentEnum::Bytes(_) => "NodeContent::Bytes(...)".to_string(),
    //         NodeContentEnum::String(s) => format!("NodeContent::String({})", s),
    //         NodeContentEnum::Nodes(n) => format!("NodeContent::Nodes(len={})", n.len()),
    //     }
    // }
}


#[pyclass]
pub struct Attrs {
    #[pyo3(get, set)]
    pub key: String,
    #[pyo3(get, set)]
    pub value: pyo3::Py<PyList>, // PyList<NodeValue>

}
#[pyclass]
pub struct Node {
    #[pyo3(get, set)]
    pub tag: String,
    #[pyo3(get, set)]
    pub attrs: Py<PyList>, // PyList<Attrs>
    #[pyo3(get, set)]
    pub content: Py<PyAny>, // Py<PyAny> yang bisa berisi NodeContent atau None
}

#[pymethods]
impl Node {
    #[new]
    pub fn new(tag: String, attrs: Option<Py<PyList>>, content: Option<pyo3::Py<NodeContent>>) -> PyResult<Self> {
        Python::attach(|py| {
            let content_py = match content {
                Some(c) => c.into_any(),
                None => py.None(),
            };
            let attrs_py = match attrs {
                Some(a) => {
                    let list = a.clone_ref(py);
                    for item in list.bind(py).iter() {
                        let _ = item.extract::<Py<Attrs>>()?;
                    };
                    list
                },
                None => PyList::empty(py).unbind(),
            };
            Ok(Self { tag, attrs: attrs_py, content: content_py })
        })
    }

    // pub fn __repr__(&self, py: Python<'_>) -> String {
    //     match &self.content {
    //         Some(c) => {
    //             let content_ref = c.bind(py).borrow();
    //             format!("Node(tag={}, content={})", self.tag, content_ref.__repr__())
    //         }
    //         None => format!("Node(tag={}, content=None)", self.tag),
    //     }
    // }
}
impl Node {
    pub fn to_node_builder(&self, py: Python<'_>) -> NodeBuilder {
        let mut builder = NodeBuilder::new(self.tag.clone());
        if let Some(content) = &self.content {
            let content_ref = content.bind(py).borrow();
            match &content_ref.inner {
                NodeContentEnum::Bytes(b) => {
                    let b_extract = b.extract::<Vec<u8>>(py).unwrap();
                    builder = builder.bytes(b_extract);
                }
                NodeContentEnum::String(s) => {
                    let s_extract = s.to_str(py).unwrap();
                    builder = builder.string_content(s_extract);
                }
                NodeContentEnum::Nodes(n) => {
                    n.bind(py).iter().for_each(|item| {
                        let it= item.extract();
                    });
                }
            }
        }
        builder
    }
    pub fn from_node_builder(builder: NodeBuilder) -> Self {
        let node = builder.build();
        Self { tag: node.tag, content: node.content }

    }
}