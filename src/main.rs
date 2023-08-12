use tokio::process::Command;

struct Node {
    addr: String,
    name: Option<String>,
}

impl Node {
    fn afficher(&self) {
        match self.name.clone() {
            Some(name) => println!("{:<15} {}", self.addr, name),
            None => println!("{}", self.addr),
        }
    }
}

struct IpAddr {
    addr: u32,
    mask: Option<u8>,
}

impl IpAddr {
    fn new(addr: u32) -> IpAddr {
        IpAddr { addr, mask: None }
    }

    fn from(value: String) -> IpAddr {
        let addr_mask: Vec<_> = value.split("/").collect();
        let mut mask = match addr_mask.len() {
            0 => unreachable!(),
            1 => None,
            2 => Some(addr_mask[1].parse().expect("le masque n'est pas un entier 8 bits")),
            _ => panic!("plusieurs masques passés pour une adresse ip"),
        };

        let addr = addr_mask[0];

        let membres: Vec<_> = addr.split(".").collect();

        if !membres.len()==4 {
            panic!("nombre de membres invalide");
        }

        let mut addr = 0u32;

        let mut val = 0;

        for membre in membres {
            let empty = membre.len() == 0;
            let membre: u8 = membre.parse().unwrap_or(0);
            addr = addr * 256 + membre as u32;

            if empty {
                if mask.is_none() {
                    mask = Some(val);
                }
            }

            val += 8;
        }

        IpAddr { addr, mask }
    }

    fn to_string(&self) -> String {
        format!(
            "{}.{}.{}.{}",
            (self.addr & 0xff000000) >> 24,
            (self.addr & 0x00ff0000) >> 16,
            (self.addr & 0x0000ff00) >> 8,
            (self.addr & 0x000000ff),
        )
    }
}

async fn ping(addr: &str) -> bool {
    //println!("Ping {}", addr);
    match Command::new("ping")
        .arg("-W")
        .arg("1")
        .arg("-w")
        .arg("1")
        .arg("-c")
        .arg("1")
        .arg("-q")
        .arg(addr)
        .output()
        .await
    {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

async fn nslookup(addr: &str) -> Option<String> {
    //println!("Look {}", addr);
    match Command::new("nslookup")
        .arg(addr)
        .output()
        .await
    {
        Ok(out) => {
            if !out.status.success() {
                return None;
            }

            let stdout = String::from_utf8(out.stdout.into())
                .unwrap_or(String::new());

            if !stdout.contains("name = ") {
                return None;
            }

            let debut = stdout.find("name = ").expect("impossible de determiner le debut") + 7;

            let stdout = String::from(&stdout[debut..stdout.len()]);

            if !stdout.contains(".\n") {
                return None;
            }

            let fin = stdout.find(".\n").expect("impossible de determiner la fin");

            Some(String::from(&stdout[0..fin]))
        },
        Err(_) => None,
    }
}

async fn identify(addr: String) -> Option<Node> {
    if ping(&addr).await {
        Some(Node { name: nslookup(&addr).await, addr })
    } else {
        None
    }
}

async fn scan(addrs: Vec<u32>) {
    let mut tasks: Vec<_> = vec![];
    for addr in addrs {
        tasks.push(tokio::spawn(identify(IpAddr::new(addr).to_string())));
    }

    for handle in tasks {
        match handle.await.unwrap_or(None) {
            Some(node) => node.afficher(),
            None => (),
        }
    }
}

fn parse_arg(arg: String) -> Vec<u32> {
    if arg.contains("-") {
        return parse_range(arg);
    }
    parse_simple(arg)
}

fn parse_range(arg: String) -> Vec<u32> {
    let pos = arg.find("-").unwrap_or(0);
    let debut = IpAddr::from(arg[0..pos].to_string()).addr;
    let fin = IpAddr::from(arg[pos+1..arg.len()].to_string()).addr;

    let mut addrs = vec![];

    for addr in debut..=fin {
        addrs.push(addr);
    }

    addrs
}

fn parse_simple(arg: String) -> Vec<u32> {
    if arg.len() == 0 {
        panic!("argument vide")
    }

    let ip_addr = IpAddr::from(arg);

    match ip_addr.mask {
        Some(mask) => {
            let mut addrs = vec![];
            let ipmask: u32 = match mask {
                0 => 0xffffffff,
                mask => 0xffffffff - (2u32.pow(32 - mask as u32) - 1),
            };
            let base = ip_addr.addr & ipmask;

            let fin = match mask {
                0 => 0xffffffff,
                mask => 2u32.pow(32 - mask as u32) - 1,
            };

            for i in 0..fin {
                addrs.push(base + i);
            }

            addrs
        },
        None => {
            vec![ip_addr.addr]
        },
    }
}

async fn get_interfaces() -> Vec<String> {
    match Command::new("bash")
        .arg("-c")
        .arg("ip -o addr | awk '!/^[0-9]*: ?lo|link\\/ether/ {print $4}' | grep -v :")
        .output()
        .await
    {
        Ok(out) => {
            let mut ifaces = vec![];

            for addr in String::from_utf8(out.stdout).unwrap_or_default().split("\n") {
                if addr.len() > 0 {
                    ifaces.push(addr.to_string());
                }
            }

            ifaces
        },
        Err(e) => {
            println!("{}", e);
            vec![]
        },
    }
}

#[tokio::main]
async fn main() {
    let mut addrs: Vec<u32> = vec![];

    match std::env::args().len() {
        0 => unreachable!(),
        1 => {
            for addr in get_interfaces().await {
                addrs.append(&mut parse_simple(addr))
            }
        },
        n => {
            let args: Vec<String> = std::env::args().collect();

            for i in [1..n] {
                addrs.append(&mut parse_arg(args[i].concat()))
            }
        },
    }

    scan(addrs).await;
}
