#[derive(Debug, Clone)]
/// MGW identifier in `ip:port` format.
pub struct MgwId {
    pub ip: [u8; 4],
    pub port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Generic format error type.
pub struct FormatError;

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("FormatError")
    }
}

impl std::error::Error for FormatError {}

impl std::str::FromStr for MgwId {
    type Err = FormatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((ip, port)) = s.split_once(':') {
            let port: u16 = port.parse().map_err(|_| FormatError)?;
            let ip = ip
                .split('.')
                .map(|d| d.parse().map_err(|_| FormatError))
                .collect::<Result<Vec<u8>, _>>()?;
            let ip = ip.try_into().map_err(|_| FormatError)?;
            Ok(Self { ip, port })
        } else {
            Err(FormatError)
        }
    }
}

impl std::fmt::Display for MgwId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}:{}",
            self.ip[0], self.ip[1], self.ip[2], self.ip[3], self.port
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// N7 signaling point code represented by three bytes.
pub struct N7SPC([u8; 3]);

impl std::str::FromStr for N7SPC {
    type Err = FormatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 6 {
            return Err(FormatError);
        }
        let ni = u8::from_str_radix(&s[..2], 16).map_err(|_| FormatError)?;
        let pc1 = u8::from_str_radix(&s[2..4], 16).map_err(|_| FormatError)?;
        let pc2 = u8::from_str_radix(&s[4..], 16).map_err(|_| FormatError)?;
        Ok(Self([ni, pc1, pc2]))
    }
}

impl std::fmt::Display for N7SPC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}{:X}{:X}", self.0[0], self.0[1], self.0[2])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// AGCF IPv4 address value.
pub struct AgcfIpaddr([u8; 4]);

impl std::str::FromStr for AgcfIpaddr {
    type Err = FormatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ip = s
            .split('.')
            .map(|d| d.parse().map_err(|_| FormatError))
            .collect::<Result<Vec<u8>, _>>()?;
        let ip = ip.try_into().map_err(|_| FormatError)?;
        Ok(Self(ip))
    }
}

impl std::fmt::Display for AgcfIpaddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}.{}", self.0[0], self.0[1], self.0[2], self.0[3])
    }
}