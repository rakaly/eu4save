use s3::{creds::Credentials, region::Region, bucket::Bucket};
use std::path::Path;
use std::fs;

pub fn request<S: AsRef<str>>(input: S) -> Vec<u8> {
    let reffed = input.as_ref(); 
    let cache = Path::new("assets").join("saves").join(reffed);
    if cache.exists() {
        println!("cache hit: {}", reffed);
        fs::read(cache).unwrap()
    } else {
        println!("cache miss: {}", reffed);
        let bucket_name = "eu4saves-test-cases";
        let region_name = "us-west-002".to_string();
        let endpoint = "s3.us-west-002.backblazeb2.com".to_string();
        let region = Region::Custom { region: region_name, endpoint };
        let credentials = Credentials::anonymous().unwrap();
        let bucket = Bucket::new(bucket_name, region, credentials).unwrap();
        let (data, code) = bucket.get_object_blocking(reffed).unwrap();

        if code != 200 {
            panic!("expected a 200 code from s3");
        } else {
            fs::create_dir_all(cache.parent().unwrap()).unwrap();
            fs::write(cache, &data).unwrap();
            data
        }
    }
}
