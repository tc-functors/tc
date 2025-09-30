use serde_derive::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Attribute {
    pub name: String,
    pub rtype: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KeySchema {
    pub name: String,
    pub rtype: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TableSpec {
    pub name: String,
    pub attributes: Vec<Attribute>,
    pub key_schema: Vec<KeySchema>
}


// impl TableSpec {

//     fn new() -> TableSpec {



//     }

// }
