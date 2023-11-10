use ft::net::SocketAddr;

#[test]
fn ipv6() {
    let addr = SocketAddr::V6([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], 1234);
    assert_eq!(addr.to_string(), "[257:257:257:257:257:257:257:257]:1234");
}

#[test]
fn ipv6_gap() {
    let addr = SocketAddr::V6([1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1], 1234);
    assert_eq!(addr.to_string(), "[257::257:257:257:257]:1234");
}

#[test]
fn ipv6_2_gaps() {
    let addr = SocketAddr::V6([1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 1, 1], 1234);
    assert_eq!(addr.to_string(), "[257::257:257:0:257]:1234");
}

#[test]
fn ipv6_gap_end() {
    let addr = SocketAddr::V6([1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0], 1234);
    assert_eq!(addr.to_string(), "[257:257:257:0:257::]:1234");
}

#[test]
fn ipv6_gap_start() {
    let addr = SocketAddr::V6([0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 1, 1], 1234);
    assert_eq!(addr.to_string(), "[::257:257:0:257]:1234");
}

#[test]
fn ipv6_zero() {
    let addr = SocketAddr::V6([0; 16], 1234);
    assert_eq!(addr.to_string(), "[::]:1234");
}
