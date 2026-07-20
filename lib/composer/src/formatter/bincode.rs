use crate::Topology;
use kit as u;

pub fn pprint(topology: &Topology) {
    let byea: Vec<u8> = bincode::serialize(topology).unwrap();
    let path = format!("{}-{}.tc", topology.namespace, topology.version);
    u::write_bytes(&path, byea);
    println!("Wrote {} ({})", &path, u::file_size_human(u::file_size(&path)));
}
