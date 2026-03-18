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
    Bytes(Py<Vec<u8>>),
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
            NodeContentEnum::Bytes(b) => b.clone_ref(py).into_any(),
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
    pub value: Vec<pyo3::Py<NodeValue>>, // PyList<NodeValue>

}
#[pymethods]
impl Attrs {
    #[new]
    fn new(key: String, value: Vec<Py<NodeValue>>) -> Self {
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
        let mut builder = NodeBuilder::new(self.tag.clone());

        if let Some(content) = &self.content {
            let content_ref = content.bind(py).borrow();
            match &content_ref.inner {
                NodeContentEnum::Bytes(b) => {
                    let b_extract = b.extract::<Vec<u8>>(py).unwrap();
                    builder = builder.bytes(b_extract);
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

        for attr in &self.attrs {
            let attr_ref = attr.bind(py).borrow();
            if attr_ref.value.len() == 1 {
                let value_ref = attr_ref.value[0].bind(py).borrow();
                builder = match &value_ref.inner {
                    NodeValueEnum::String(s) => builder.attr(attr_ref.key.clone(), s.clone()),
                    NodeValueEnum::Jid(jid) => {
                        let jid_ref = jid.bind(py).borrow();
                        builder.jid_attr(attr_ref.key.clone(), jid_ref.as_whatsapp_jid())
                    }
                };
            } else {
                let joined = attr_ref
                    .value
                    .iter()
                    .map(|v| {
                        let v_ref = v.bind(py).borrow();
                        match &v_ref.inner {
                            NodeValueEnum::String(s) => s.clone(),
                            NodeValueEnum::Jid(jid) => {
                                let jid_ref = jid.bind(py).borrow();
                                jid_ref.as_whatsapp_jid().to_string()
                            }
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                builder = builder.attr(attr_ref.key.clone(), joined);
            }
        }

        builder
    }

    pub fn from_node_builder(builder: NodeBuilder) -> Self {
        let node = builder.build();

        Self {
            tag: node.tag,
            attrs: Vec::new(),
            content: None,
        }
    }

    pub fn from_node(node: wacore_binary::node::Node) -> Self {
        //
    }
}