use crate::config::{
    Config,
    StoreMode,
};
use composer::{
    Channel,
    Event,
    Function,
    Mutation,
    Route,
    Topology,
};
use serde::Deserialize;
use std::{
    collections::HashMap,
    sync::Arc,
};
use surrealdb::{
    Surreal,
    engine::local::{
        Db,
        Mem,
    },
    opt::RecordId,
};
use tokio::sync::Mutex;

#[derive(Debug, Deserialize)]
struct Record {
    #[allow(dead_code)]
    id: RecordId,
}

#[derive(Clone)]
pub struct Store {
    pub db: Arc<Mutex<Surreal<Db>>>,
}
pub const TOPOLOGY_TABLE: &str = "topology";

impl Store {
    pub async fn new(config: Config) -> Store {
        // TODO: take db connection from config or param
        let db = match config.store_mode {
            StoreMode::Mem => Surreal::new::<Mem>(()).await.unwrap(),
            _ => panic!("No StoreMode configured"),
        };
        db.use_ns("test").use_db("test").await.unwrap();
        Store {
            db: Arc::new(Mutex::new(db)),
        }
    }

    pub async fn load(&self, topologies: HashMap<String, Topology>) -> surrealdb::Result<()> {
        let db = self.db.lock().await;

        for (_, topology) in topologies {
            let _created: Option<Record> = db
                .create((TOPOLOGY_TABLE, &topology.namespace))
                .content(topology)
                .await?;
            //dbg!(created);
        }

        Ok(())
    }

    pub async fn list_topologies(&self) -> Vec<Topology> {
        let db = self.db.lock().await;
        let topologies: Vec<Topology> = db.select(TOPOLOGY_TABLE).await.unwrap();
        topologies
    }

    pub async fn find_topology(&self, root: &str, namespace: &str) -> Option<Topology> {
        let db = self.db.lock().await;
        let maybe_topology: Option<Topology> = db.select((TOPOLOGY_TABLE, root)).await.unwrap();
        if root == namespace {
            maybe_topology
        } else {
            match maybe_topology {
                Some(t) => t.nodes.get(namespace).cloned(),
                None => None,
            }
        }
    }

    pub async fn find_events(&self, root: &str, namespace: &str) -> HashMap<String, Event> {
        let rt = self.find_topology(root, namespace).await;
        match rt {
            Some(t) => t.events.clone(),
            None => HashMap::new(),
        }
    }

    pub async fn find_functions(&self, root: &str, namespace: &str) -> HashMap<String, Function> {
        let rt = self.find_topology(root, namespace).await;
        match rt {
            Some(t) => {
                let mut h: HashMap<String, Function> = t.functions.clone();
                for (_, node) in t.nodes {
                    h.extend(node.functions);
                }
                h
            }
            None => HashMap::new(),
        }
    }

    pub async fn find_routes(&self, root: &str, namespace: &str) -> HashMap<String, Route> {
        let rt = self.find_topology(root, namespace).await;
        match rt {
            Some(t) => t.routes.clone(),
            None => HashMap::new(),
        }
    }

    pub async fn find_channels(&self, root: &str, namespace: &str) -> HashMap<String, Channel> {
        let rt = self.find_topology(root, namespace).await;
        match rt {
            Some(t) => t.channels.clone(),
            None => HashMap::new(),
        }
    }

    pub async fn find_mutations(&self, root: &str, namespace: &str) -> HashMap<String, Mutation> {
        let rt = self.find_topology(root, namespace).await;
        match rt {
            Some(t) => t.mutations.clone(),
            None => HashMap::new(),
        }
    }

    pub async fn find_function(&self, root: &str, namespace: &str, id: &str) -> Option<Function> {
        let fns = self.find_functions(root, namespace).await;
        fns.get(id).cloned()
    }

    pub async fn find_all_events(&self) -> HashMap<String, Event> {
        let topologies = self.list_topologies().await;
        let mut h: HashMap<String, Event> = HashMap::new();
        for node in topologies {
            h.extend(node.events);
            for (_, n) in node.nodes {
                h.extend(n.events);
            }
        }
        h
    }

    pub async fn find_root_namespaces(&self) -> Vec<String> {
        let ts = self.list_topologies().await;
        let mut xs: Vec<String> = vec![];
        for t in ts {
            if !t.events.is_empty() {
                xs.push(t.namespace)
            }
        }
        xs
    }
}
