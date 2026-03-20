use pyo3::types::PyString;
use pyo3::{Python, pyclass, pymethods};
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
    Bytes(Vec<u8>),
    String(String),
    Nodes(Vec<Py<Node>>), // PyList<Node>
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
        let result = match &self.inner {
            NodeContentEnum::Bytes(b) => b.into_py_any(py).unwrap(),
            NodeContentEnum::String(s) => s.into_py_any(py).unwrap(),
            NodeContentEnum::Nodes(n) => n.into_py_any(py).unwrap(),
        };
        result
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

    pub fn __repr__(&self, _py: Python<'_>) -> PyResult<String> {
        let repr = match &self.inner {
            NodeContentEnum::Bytes(_) => "NodeContent::Bytes(...)".to_string(),
            NodeContentEnum::String(s) => format!("NodeContent::String({})", s),
            NodeContentEnum::Nodes(n) => format!("NodeContent::Nodes(len={})", n.len()),
        };
        Ok(repr)
    }
}


#[pyclass]
pub struct Attrs {
    #[pyo3(get, set)]
    pub key: String,
    #[pyo3(get, set)]
    pub value: pyo3::Py<NodeValue>, // PyList<NodeValue>

}
#[pymethods]
impl Attrs {
    #[new]
    fn new(key: String, value: Py<NodeValue>) -> Self {
        Attrs {
            key: key,
            value: value
        }
    }
}
#[pyclass]
pub struct Node {
    #[pyo3(get, set)]
    pub tag: String,
    #[pyo3(get, set)]
    pub attrs: Vec<Py<Attrs>>, // PyList<Attrs>
    #[pyo3(get, set)]
    pub content: Option<Py<NodeContent>>, // Py<PyAny> yang bisa berisi NodeContent atau None
}

#[pymethods]
impl Node {
    #[new]
    pub fn new(tag: String, attrs: Vec<Py<Attrs>>, content: Option<pyo3::Py<NodeContent>>) -> Self {
        Self { tag, attrs: attrs, content: content }
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
        let mut builder = NodeBuilder::new_dynamic(self.tag.clone());

        // Handle content if present
        if let Some(content) = &self.content {
            let content_ref = content.bind(py).borrow();
            match &content_ref.inner {
                NodeContentEnum::Bytes(b) => {
                    builder = builder.bytes(b.clone());
                }
                NodeContentEnum::String(s) => {
                    builder = builder.string_content(s.clone());
                }
                NodeContentEnum::Nodes(n) => {
                    let nodes_extract = n
                        .iter()
                        .map(|node| {
                            let node_ref = node.bind(py).borrow();
                            node_ref.to_node_builder(py).build()
                        })
                        .collect::<Vec<_>>();
                    builder = builder.children(nodes_extract);
                }
            };
        }

        // Handle attributes
        
        for attr in &self.attrs {
            let attr_ref = attr.bind(py).borrow();
            let value_ref = attr_ref.value.bind(py).borrow();

            // Ensure the key lives long enough for the builder which expects '&'static str'.
            // Minimal overhead: leak the owned String (intentional small leak for static lifetime).
            let key_owned = attr_ref.key.clone();
            let key_static: &'static str = Box::leak(key_owned.into_boxed_str());

            builder = match &value_ref.inner {
                NodeValueEnum::String(s) => builder.attr(key_static, s.clone()),
                NodeValueEnum::Jid(jid) => {
                    let jid_ref = jid.bind(py).borrow();
                    builder.attr(key_static, jid_ref.__repr__()) // Use appropriate JID representation
                }
            };
        }

        builder
    }

    pub fn from_node(node: &wacore_binary::node::Node) -> Self {
        let attrs = Python::attach(|py| {
            node.attrs
                .iter()
                .map(|(k, v)| {
                    let value = match v {
                        wacore_binary::node::NodeValue::String(s) => NodeValue::new_string(s.clone()),
                        wacore_binary::node::NodeValue::Jid(jid) => {
                            NodeValue::jid(Py::new(py, JID::from(jid.clone())).unwrap())
                        }
                    };
                    Py::new(py, Attrs::new(k.to_string(), Py::new(py, value).unwrap())).unwrap()
                })
                .collect::<Vec<_>>()
        });

        let content = node.content.as_ref().map(|c| {
            Python::attach(|py| {
                let content_enum = match c {
                    wacore_binary::node::NodeContent::Bytes(b) => NodeContentEnum::Bytes(b.clone()),
                    wacore_binary::node::NodeContent::String(s) => NodeContentEnum::String(s.clone()),
                    wacore_binary::node::NodeContent::Nodes(n) => {
                        let nodes = n
                            .iter()
                            .map(Self::from_node)
                            .map(|node| Py::new(py, node).unwrap())
                            .collect();
                        NodeContentEnum::Nodes(nodes)
                    }
                };
                Py::new(py, NodeContent { inner: content_enum }).unwrap()
            })
        });

        Self {
            tag: node.tag.to_string(),
            attrs,
            content,
        }
    }
}