//! Print the local replica status as read by ic-agent: `cargo run -p apparatus-ic --example ic-status [URL]`.

fn main() {
    let url = std::env::args()
        .nth(1)
        .unwrap_or_else(|| apparatus_ic::DEFAULT_URL.to_string());
    println!("ic-agent {}", apparatus_ic::IC_AGENT_VERSION);
    println!("url {}", apparatus_ic::status_url(&url));
    match apparatus_ic::status(&url) {
        Ok(s) => {
            println!(
                "replica_health_status {}",
                s.health.as_deref().unwrap_or("-")
            );
            println!(
                "certified_height {}",
                s.certified_height.map_or("-".into(), |h| h.to_string())
            );
            println!("impl_version {}", s.impl_version.as_deref().unwrap_or("-"));
            println!(
                "root_key_sha256 {}",
                s.root_key_sha256.as_deref().unwrap_or("-")
            );
            println!(
                "root_key_principal {}",
                s.root_key_principal.as_deref().unwrap_or("-")
            );
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(if e.is_unreachable() { 3 } else { 1 });
        }
    }
}
