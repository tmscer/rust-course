use std::{fs, io, path};

use rustls_pemfile::{certs, rsa_private_keys};

// No real reason to make this async since this runs once at startup and
// functions from `rustls_pemfile` are synchronous.

pub fn load_root_certs(path: &path::Path) -> anyhow::Result<rustls::RootCertStore> {
    let mut root_store = rustls::RootCertStore::empty();

    for cert in load_certs(path)? {
        root_store.add(cert)?;
    }

    Ok(root_store)
}

pub fn load_certs(path: &path::Path) -> io::Result<Vec<rustls_pki_types::CertificateDer<'static>>> {
    certs(&mut io::BufReader::new(fs::File::open(path)?)).collect()
}

pub fn load_keys(path: &path::Path) -> io::Result<rustls_pki_types::PrivateKeyDer<'static>> {
    rsa_private_keys(&mut io::BufReader::new(fs::File::open(path)?))
        .next()
        .unwrap()
        .map(Into::into)
}
