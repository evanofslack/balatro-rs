use std::path::Path;

use plist::Value;

use crate::error::Error;
use crate::paths::Paths;

/// Reads the binary plist and extracts the meta.jkr and profile.jkr blobs
/// for the given profile. Returns (meta_bytes, profile_bytes).
pub fn extract(paths: &Paths) -> Result<(Vec<u8>, Vec<u8>), Error> {
    let plist = load_plist(&paths.plist)?;
    let dict = plist.as_dictionary().ok_or(Error::PlistNotDict)?;

    let meta = extract_blob(dict, &paths.meta_key())?;
    let profile = extract_blob(dict, &paths.profile_key())?;

    Ok((meta, profile))
}

/// Reads the binary plist and injects updated meta.jkr and profile.jkr blobs
/// for the given profile, then writes the plist back.
pub fn inject(paths: &Paths, meta: Vec<u8>, profile: Vec<u8>) -> Result<(), Error> {
    let plist = load_plist(&paths.plist)?;
    let mut dict = plist.into_dictionary().ok_or(Error::PlistNotDict)?;

    dict.insert(paths.meta_key(), Value::Data(meta));
    dict.insert(paths.profile_key(), Value::Data(profile));

    Value::Dictionary(dict)
        .to_file_binary(&paths.plist)
        .map_err(|e| Error::PlistWrite(e.to_string()))?;

    Ok(())
}

/// Returns which profile numbers (1–3) have blobs present in the plist.
pub fn present_profiles(plist_path: &Path) -> Result<Vec<u8>, Error> {
    let plist = load_plist(plist_path)?;
    let dict = plist.as_dictionary().ok_or(Error::PlistNotDict)?;

    let profiles = (1u8..=3)
        .filter(|n| dict.contains_key(&format!("{n}__profile.jkr.data")))
        .collect();

    Ok(profiles)
}

fn load_plist(path: &Path) -> Result<Value, Error> {
    if !path.exists() {
        return Err(Error::PlistNotFound(path.to_path_buf()));
    }
    Value::from_file(path).map_err(|e| Error::PlistRead(e.to_string()))
}

fn extract_blob(dict: &plist::Dictionary, key: &str) -> Result<Vec<u8>, Error> {
    match dict.get(key) {
        Some(Value::Data(bytes)) => Ok(bytes.clone()),
        Some(_) => Err(Error::PlistKeyWrongType(key.to_string())),
        None => Err(Error::PlistKeyMissing(key.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::Platform;

    #[test]
    fn inject_then_extract_roundtrips() {
        let plist =
            std::env::temp_dir().join(format!("balatro-storage-test-{}.plist", std::process::id()));
        Value::Dictionary(plist::Dictionary::new())
            .to_file_binary(&plist)
            .unwrap();
        let paths = Paths::new(Platform::MacOsArcade, 3)
            .unwrap()
            .with_plist(Some(plist.clone()));

        inject(&paths, b"meta".to_vec(), b"profile".to_vec()).unwrap();

        assert_eq!(
            extract(&paths).unwrap(),
            (b"meta".to_vec(), b"profile".to_vec())
        );
        assert_eq!(present_profiles(&plist).unwrap(), vec![3]);
        std::fs::remove_file(plist).unwrap();
    }
}
