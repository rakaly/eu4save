use std::fs::File;
use std::path::Path;
use std::sync::{LazyLock, Mutex};

static DATA: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

/// Fetch an eu4 save file. Save files can be quite large, so the save files are not stored in the
/// repo. Instead they are stored in a public S3 bucket. This function will check if the file has
/// been cached, else fetch it from the S3 bucket. Previous implementations used git lfs, but had
/// to be migrated away as we ran out of the monthly free bandwidth (1GB) on day 1 (and even git
/// lfs caching was used). The S3 implementation used is backblaze, which provides 1GB free
/// download per day, so I'm not anticipating paying more than a few cents a year to maintain this
/// repository of saves.
pub fn request_file<S: AsRef<str>>(input: S) -> File {
    let reffed = input.as_ref();
    let cache = Path::new("assets").join("saves").join(reffed);
    if cache.exists() {
        println!("cache hit: {}", reffed);
    } else {
        let guard = DATA.lock().unwrap();
        if cache.exists() {
            drop(guard);
            println!("cache hit: {}", reffed);
        } else {
            println!("cache miss: {}", reffed);
            let url = format!("https://cdn-dev.pdx.tools/eu4-saves/{}", reffed);
            let mut resp = attohttpc::get(&url).send().unwrap();

            if !resp.is_success() {
                panic!(
                    "expected a 200 code from s3, but received {}",
                    resp.status()
                );
            } else {
                std::fs::create_dir_all(cache.parent().unwrap()).unwrap();
                let mut f = std::fs::File::create(&cache).unwrap();
                std::io::copy(&mut resp, &mut f).unwrap();
            }
        }
    }

    std::fs::File::open(cache).unwrap()
}
