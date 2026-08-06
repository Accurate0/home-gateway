use esp_idf_svc::ipv4::{IpInfo, Ipv4Addr, Mask, Subnet};

const MAX_WAKES_BETWEEN_DHCP: u32 = 24;

#[derive(Clone, Copy)]
pub struct NetCache {
    valid: bool,
    wakes_since_dhcp: u32,
    bssid: [u8; 6],
    channel: u8,
    ip: [u8; 4],
    gateway: [u8; 4],
    mask: u8,
    dns: Option<[u8; 4]>,
}

impl NetCache {
    const EMPTY: Self = Self {
        valid: false,
        wakes_since_dhcp: 0,
        bssid: [0; 6],
        channel: 0,
        ip: [0; 4],
        gateway: [0; 4],
        mask: 0,
        dns: None,
    };

    pub fn bssid(&self) -> [u8; 6] {
        self.bssid
    }

    pub fn channel(&self) -> u8 {
        self.channel
    }

    pub fn ip(&self) -> Ipv4Addr {
        Ipv4Addr::from(self.ip)
    }

    pub fn subnet(&self) -> Subnet {
        Subnet {
            gateway: Ipv4Addr::from(self.gateway),
            mask: Mask(self.mask),
        }
    }

    pub fn dns(&self) -> Option<Ipv4Addr> {
        self.dns.map(Ipv4Addr::from)
    }
}

#[link_section = ".rtc.data"]
static mut CACHE: NetCache = NetCache::EMPTY;

fn read() -> NetCache {
    unsafe { (&raw const CACHE).read() }
}

fn write(cache: NetCache) {
    unsafe { (&raw mut CACHE).write(cache) }
}

pub fn take() -> Option<NetCache> {
    let mut cache = read();

    if !cache.valid {
        return None;
    }

    if cache.wakes_since_dhcp >= MAX_WAKES_BETWEEN_DHCP {
        log::info!("renewing dhcp lease after {} wakes", cache.wakes_since_dhcp);
        invalidate();
        return None;
    }

    cache.wakes_since_dhcp = cache.wakes_since_dhcp.saturating_add(1);
    write(cache);

    Some(cache)
}

pub fn store(bssid: [u8; 6], channel: u8, ip_info: &IpInfo) {
    write(NetCache {
        valid: true,
        wakes_since_dhcp: 0,
        bssid,
        channel,
        ip: ip_info.ip.octets(),
        gateway: ip_info.subnet.gateway.octets(),
        mask: ip_info.subnet.mask.0,
        dns: ip_info.dns.map(|dns| dns.octets()),
    });

    log::info!(
        "cached association: bssid {bssid:02x?} channel {channel} ip {}",
        ip_info.ip
    );
}

pub fn invalidate() {
    write(NetCache::EMPTY);
}
