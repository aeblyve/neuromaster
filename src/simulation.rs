use domain::base::Dname;
use fdg_sim::petgraph::graph::NodeIndex;
use fdg_sim::{ForceGraph, ForceGraphHelper, Simulation, SimulationParameters};
use kiss3d::nalgebra::{Point3, Vector3};
use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Clone, Debug, PartialEq)]
pub struct SimpleHost {
    pub main_addr: IpAddr,
    pub main_hostname: Option<Dname<Vec<u8>>>,
    pub os_guess: Option<OsGuess>,
    pub rtt: Option<f32>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OsGuess {
    LINUX(String),
    FREEBSD(String),
    OPENBSD(String),
    OTHER(String),
}

impl OsGuess {
    // This works fine if nobody trips LinuxFreeBSDOpenBSD in the nmap database :P
    fn from_string(string: &String) -> OsGuess {
        if string.contains("Linux") {
            OsGuess::LINUX(string.to_string())
        } else if string.contains("FreeBSD") {
            OsGuess::FREEBSD(string.to_string())
        } else if string.contains("OpenBSD") {
            OsGuess::OPENBSD(string.to_string())
        } else {
            OsGuess::OTHER(string.to_string())
        }
    }
}

impl SimpleHost {
    pub fn from_fullhost(host: &rust_nmap::host) -> Result<Self, Box<dyn std::error::Error>> {
        let status = host
            .status
            .as_ref()
            .ok_or("Host has no status.")?
            .state
            .as_ref()
            .ok_or("Host status has no state")?;

        match status.as_str() {
            "down" => return Err("Fullhost is down".into()),
            _ => {}
        }

        let address_box = host
            .address
            .as_ref()
            .ok_or("No host address spec")?
            .into_iter()
            .next()
            .ok_or("No host address")?;

        let addrtype = address_box
            .addrtype
            .as_ref()
            .ok_or("No addrtype in address")?;
        let addr_str = address_box.addr.as_ref().ok_or("No addr in address")?;

        let addr = match addrtype.as_str() {
            "ipv4" => IpAddr::V4(addr_str.parse()?),
            "ipv6" => IpAddr::V6(addr_str.parse()?),
            _ => panic!("Unhandled addrtype. Stopping."),
        };

        let hostname = (|| {
            host.hostnames
                .as_ref()?
                .hostname
                .as_ref()?
                .first()?
                .name
                .as_ref()
        })();

        let hostname = hostname.map(|hostname| Dname::from_chars(hostname.chars()).unwrap());

        let os = (|| host.os.as_ref()?.osmatch.as_ref()?.first()?.name.as_ref())();

        let os = os.map(|os| OsGuess::from_string(os));

        Ok(Self {
            main_addr: addr,
            main_hostname: hostname,
            os_guess: os,
            rtt: None,
        })
    }

    pub fn from_hop(hop: &rust_nmap::hop) -> Result<Self, Box<dyn std::error::Error>> {
        let hostname = match hop.host.as_ref() {
            Some(_) => Some(Dname::<Vec<u8>>::from_chars(
                hop.host.as_ref().unwrap().chars(),
            )?),
            None => None,
        };

        Ok(Self {
            main_addr: hop.ipaddr.as_ref().unwrap().parse().unwrap(),
            main_hostname: hostname,
            os_guess: None,
            rtt: None,
        })
    }

    pub fn from_strs(addr: &str, hostname: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            main_addr: addr.parse()?,
            main_hostname: Some(Dname::from_chars(hostname.chars())?),
            os_guess: None,
            rtt: None,
        })
    }

    pub fn set_rtt(host: &mut Self, rtt: f32) {
        host.rtt = Some(rtt);
    }
}

pub fn build_simulation(
    scan: rust_nmap::nmap_run,
) -> Result<Simulation<SimpleHost, ()>, Box<dyn std::error::Error>> {
    let mut map = HashMap::<IpAddr, NodeIndex>::new();
    let mut graph: ForceGraph<SimpleHost, ()> = ForceGraph::default();

    let localhost = SimpleHost::from_strs("127.0.0.1", "localhost")?;
    let localhost_addr = localhost.main_addr;
    insert(&mut map, &mut graph, localhost);

    for host in scan.host.as_ref().ok_or("Host given as None")? {
        let main = SimpleHost::from_fullhost(host);
        match main {
            Err(e) => continue,
            _ => {}
        }
        insert(&mut map, &mut graph, main?);

        let mut origin_addr = localhost_addr;
        for hop in host
            .trace
            .as_ref()
            .ok_or("Trace was none in host")?
            .hops
            .as_ref()
            .ok_or("Hops was none in trace")?
        {
            let hop_host = SimpleHost::from_hop(hop)?;
            let hop_addr = hop_host.main_addr;

            if map.contains_key(&hop_addr) {
                let origin_index = *map
                    .get(&origin_addr)
                    .ok_or("Could not find origin in map")?;
                let index = *map.get(&hop_addr).unwrap();
                graph.add_edge(origin_index, index, ());
            } else {
                let origin_index = *map
                    .get(&origin_addr)
                    .ok_or("Could not find origin in map")?;
                let index = insert(&mut map, &mut graph, hop_host);
                graph.add_edge(origin_index, index, ());
            }
            origin_addr = hop_addr;
        }
    }

    println!("{:#?}", graph);

    fn insert(
        map: &mut HashMap<IpAddr, NodeIndex>,
        graph: &mut ForceGraph<SimpleHost, ()>,
        host: SimpleHost,
    ) -> NodeIndex {
        let name = host.main_addr.to_string();
        let addr = host.main_addr;
        let index = graph.add_force_node(name, host);
        map.insert(addr, index);
        index
    }

    Ok(Simulation::from_graph(
        graph,
        SimulationParameters::new(
            20.0,
            fdg_sim::Dimensions::Three,
            fdg_sim::force::fruchterman_reingold(3.0, 0.975),
        ),
    ))
}

/// Given a ray with origin and direction, find the closest node (modeled as a sphere centered on node.location) in the simulation intersecting the ray, if it exists.
// TODO iterate over visible scenenodes using localtransform instead?
pub fn find_closest_intersection(
    ray_origin: Point3<f32>,
    ray_direction: Vector3<f32>,
    simulation: &Simulation<SimpleHost, ()>,
) -> Option<NodeIndex> {
    let radius = 1.0; // magic number for now - might change with amount of edges?
    let graph = simulation.get_graph();

    let mut least_distance = f32::MAX;
    let mut closest_node: Option<NodeIndex> = None;

    for node_index in graph.node_indices() {
        let node_weight = graph.node_weight(node_index).unwrap();
        let sphere_center = Point3::new(
            node_weight.location.x,
            node_weight.location.y,
            node_weight.location.z,
        );
        let difference: Vector3<f32> = ray_origin - sphere_center;
        let difference_sqr = difference.dot(&difference);
        let p = ray_direction.dot(&difference);

        let determinant = p * p - difference_sqr + radius * radius;
        println!(
            "For the sphere centered at {}, determinant is {}",
            sphere_center, determinant
        );

        // ... is this safe float-wise?
        if determinant < 0.0 {
            continue; // no (real) intersection
        } else if determinant == 0.0 {
            // one intersection, log it
            let distance = ray_direction.scale(-1.0).dot(&difference);
            least_distance = least_distance.min(distance);
            if distance < least_distance {
                least_distance = distance;
                closest_node = Some(node_index);
            }
        } else {
            // two intersections, log the closest one
            let distance1 = ray_direction.scale(-1.0).dot(&difference) - determinant.sqrt();
            let distance2 = ray_direction.scale(-1.0).dot(&difference) + determinant.sqrt();
            let distance = distance1.min(distance2);

            if distance < least_distance {
                least_distance = distance;
                closest_node = Some(node_index);
            }
        }
    }
    closest_node
}
