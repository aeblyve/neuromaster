use domain::base::Dname;
use fdg_sim::{ForceGraph, ForceGraphHelper, Simulation, SimulationParameters};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SimpleHost {
    pub main_addr: IpAddr,
    pub main_hostname: Option<Dname<Vec<u8>>>,
    pub rtt: Option<f32>,
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

        let hostname = host
            .hostnames
            .as_ref()
            .ok_or("No hostnames in host")?
            .hostname
            .as_ref()
            .ok_or("No hostname iter in hostnames")?
            .into_iter()
            .next()
            .ok_or("No hostname in hostname")?
            .name
            .as_ref()
            .ok_or("Hostname name is None")?;

        Ok(Self {
            main_addr: addr,
            main_hostname: Some(Dname::from_chars(hostname.chars())?),
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
            rtt: None,
        })
    }

    pub fn from_strs(addr: &str, hostname: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            main_addr: addr.parse()?,
            main_hostname: Some(Dname::from_chars(hostname.chars())?),
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

    let max = 500;
    let mut count = 0;

    for host in scan.host.as_ref().ok_or("Host given as None")? {
        // if count > max {
        //     break;
        // }
        count += 1;
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
