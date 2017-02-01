/// Wake up once per 1000 ms to send few probes
pub const INTERVAL: u64 = 1000;

/// Number of probes to send at each interval
pub const NUM_PROBES: u64 = 10;

/// If we got any probe or report during 5 seconds, don't probe this node
pub const MIN_PROBE: u64 = 5000;

/// But if we sent no probe within 60 seconds (but were receiving reports, so
/// didn't hit 5 seconds timeout above), we should send probe anyway. This
/// allows too keep roundtrip times on both nodes reasonably up to date
pub const MAX_PROBE: u64 = 60000;

/// Num of friend nodes to send within each request, everything must fit
/// MAX_PACKET_SIZE which is capped at maximum UDP packet size (65535),
/// better if it fits single IP packet (< 1500)
pub const NUM_FRIENDS: usize = 10;

/// After we had no reports from node for 20 seconds (but we sent probe during
/// this time) we consider node to be inaccessible by it's primary IP and are
/// trying to reach it by pinging any other random IP address.
pub const PREFAIL_TIME: u64 = 20_000;


/// Maximum expected roundtrip time. We consider report failing if it's not
/// received during this time. Note, this doesn't need to be absolute ceiling
/// of RTT, and we don't do any crazy things based on the timeout, this is
/// just heuristic for pre-fail condition.
pub const MAX_ROUNDTRIP: u64 = 2000;

/// After this time we consider node failing and don't send it in friendlist.
/// Note that all nodes that where up until we marked node as failinig do know
/// the node, and do ping it. This is currently used only
pub const FAIL_TIME: u64 = 3600_000;


/// This is time after last heartbeat when node will be removed from the list
/// of known nodes. This should be long after FAIL_TIME. (But not necessarily
/// 48x longer, as we do now).
/// Also note that node will be removed from all peers after
/// FAIL_TIME + REMOVE_TIME + longest-round-trip-time
pub const REMOVE_TIME: u64 = 2 * 86400_000;

/// This is a size of our UDP buffers. The maximum value depends on NUM_FRIENDS
/// and the number of IP addresses at each node. It's always capped at maximum
/// UDP packet size of 65535
pub const MAX_PACKET_SIZE: usize = 8192;



// Expectations:
//     MAX_PROBE > MIN_PROBE
//     MAX_ROUNDTRIP <= MAX_PROBE
//     FAIL_TIME + some big value < REMOVE_TIME
