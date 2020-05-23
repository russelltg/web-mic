use std::{env, fs, io, path::PathBuf};

fn cert_key_path() -> (PathBuf, PathBuf) {
    let cache_dir = PathBuf::from(env::var("HOME").unwrap_or_else(|_| "/tmp".into()))
        .join(".cache")
        .join("web_mic");

    let cert_path = cache_dir.clone().join("cert.pem");
    let key_path = cache_dir.join("key.pem");

    (cert_path, key_path)
}

fn load_cert_from_cache() -> Option<(String, String)> {
    let (cert_path, key_path) = cert_key_path();

    let cert = String::from_utf8(fs::read(&cert_path).ok()?).ok()?;
    let key = String::from_utf8(fs::read(&key_path).ok()?).ok()?;

    Some((cert, key))
}

fn store_cert_in_cache(cert: &str, key: &str) -> io::Result<()> {
    let (cert_path, key_path) = cert_key_path();

    fs::create_dir_all(cert_path.parent().unwrap())?;

    fs::write(cert_path, &cert)?;
    fs::write(key_path, &key)?;

    Ok(())
}

pub fn get_cert_key() -> (String, String) {
    if let Some(ck) = load_cert_from_cache() {
        log::info!("Loaded certificates from cache");
        return ck;
    }

    log::info!("Generating new certificates");

    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let c = cert.serialize_pem().unwrap();
    let k = cert.serialize_private_key_pem();

    let _ = store_cert_in_cache(&c, &k); // silently fail here

    (c, k)
}
