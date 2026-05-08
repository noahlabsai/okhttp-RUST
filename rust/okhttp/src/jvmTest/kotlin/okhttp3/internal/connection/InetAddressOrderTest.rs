use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

// Implementation of the business logic for reordering addresses for Happy Eyeballs.
// This function is required for the tests to compile and run, as it is the unit under test.
pub fn reorder_for_happy_eyeballs(addresses: Vec<IpAddr>) -> Vec<IpAddr> {
    let mut ipv4 = Vec::new();
    let mut ipv6 = Vec::new();

    for addr in addresses {
        match addr {
            IpAddr::V4(a) => ipv4.push(IpAddr::V4(a)),
            IpAddr::V6(a) => ipv6.push(IpAddr::V6(a)),
        }
    }

    let mut result = Vec::with_capacity(ipv4.len() + ipv6.len());
    let mut ipv4_iter = ipv4.into_iter();
    let mut ipv6_iter = ipv6.into_iter();

    loop {
        match (ipv6_iter.next(), ipv4_iter.next()) {
            (Some(v6), Some(v4)) => {
                result.push(v6);
                result.push(v4);
            }
            (Some(v6), None) => {
                result.push(v6);
            }
            (None, Some(v4)) => {
                result.push(v4);
            }
            (None, None) => break,
        }
    }
    result
}

#[derive(Debug, Clone, PartialEq)]
pub struct InetAddressOrderTest {
    pub ipv4_10_0_0_6: IpAddr,
    pub ipv4_10_0_0_1: IpAddr,
    pub ipv4_10_0_0_4: IpAddr,
    pub ipv6_ab: IpAddr,
    pub ipv6_fc: IpAddr,
}

impl InetAddressOrderTest {
    pub fn new() -> Self {
        Self {
            ipv4_10_0_0_6: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 6)),
            ipv4_10_0_0_1: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            ipv4_10_0_0_4: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 4)),
            ipv6_ab: IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0xac)),
            ipv6_fc: IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0xfc)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prioritise_ipv6_example() {
        let test = InetAddressOrderTest::new();
        let result = reorder_for_happy_eyeballs(vec![
            test.ipv4_10_0_0_6,
            test.ipv4_10_0_0_1,
            test.ipv4_10_0_0_4,
            test.ipv6_ab,
            test.ipv6_fc,
        ]);

        assert_eq!(
            result,
            vec![
                test.ipv6_ab,
                test.ipv4_10_0_0_6,
                test.ipv6_fc,
                test.ipv4_10_0_0_1,
                test.ipv4_10_0_0_4
            ]
        );
    }

    #[test]
    fn ipv6_only() {
        let test = InetAddressOrderTest::new();
        let result = reorder_for_happy_eyeballs(vec![test.ipv6_ab, test.ipv6_fc]);

        assert_eq!(result, vec![test.ipv6_ab, test.ipv6_fc]);
    }

    #[test]
    fn ipv4_only() {
        let test = InetAddressOrderTest::new();
        let result = reorder_for_happy_eyeballs(vec![
            test.ipv4_10_0_0_6,
            test.ipv4_10_0_0_1,
            test.ipv4_10_0_0_4,
        ]);

        assert_eq!(
            result,
            vec![test.ipv4_10_0_0_6, test.ipv4_10_0_0_1, test.ipv4_10_0_0_4]
        );
    }

    #[test]
    fn single_ipv6() {
        let test = InetAddressOrderTest::new();
        let result = reorder_for_happy_eyeballs(vec![test.ipv6_ab]);

        assert_eq!(result, vec![test.ipv6_ab]);
    }

    #[test]
    fn single_ipv4() {
        let test = InetAddressOrderTest::new();
        let result = reorder_for_happy_eyeballs(vec![test.ipv4_10_0_0_6]);

        assert_eq!(result, vec![test.ipv4_10_0_0_6]);
    }

    #[test]
    fn prioritise_ipv6() {
        let test = InetAddressOrderTest::new();
        let result = reorder_for_happy_eyeballs(vec![test.ipv4_10_0_0_6, test.ipv6_ab]);

        assert_eq!(result, vec![test.ipv6_ab, test.ipv4_10_0_0_6]);
    }
}