// Copyright 2016 Drew Short <drew@sothr.com>.
//
// Licensed under the MIT license<LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

// Enable nightly features for extra testing behind the bench feature
#![cfg_attr(feature = "bench", feature(test))]

extern crate libc;
extern crate rustc_serialize;
#[cfg(feature = "bench")]
extern crate test;

pub mod hash;
pub mod cache;

use std::path::Path;
use std::ffi::CStr;
use cache::Cache;

#[repr(C)]
pub struct PIHash<'a> {
    cache: Option<Cache<'a>>,
}

impl<'a> PIHash<'a> {
    /**
     * Create a new pihash library, and initialize a cache of a path is passed.
     * If none is passed then no cache is initialized or used with the library
     */
    pub fn new(cache_path: Option<&'a str>) -> PIHash<'a> {
        match cache_path {
            Some(path) => {
                let cache = Cache {
                    cache_dir: path,
                    use_cache: true,
                };
                match cache.init() {
                    Ok(_) => PIHash { cache: Some(cache) },
                    Err(e) => {
                        println!("Error creating library with cache: {}", e);
                        PIHash { cache: None }
                    }
                }
            }
            None => PIHash { cache: None },
        }
    }

    pub fn get_perceptual_hash(&self,
                               path: &Path,
                               precision: &hash::Precision,
                               hash_type: &hash::HashType)
                               -> u64 {
        hash::get_perceptual_hash(&path, &precision, &hash_type, &self.cache)
    }

    pub fn get_phashes(&self, path: &'a Path) -> hash::PerceptualHashes {
        hash::get_perceptual_hashes(path, &hash::Precision::Medium, &self.cache)
    }


    pub fn get_ahash(&self, path: &Path) -> u64 {
        hash::get_perceptual_hash(&path,
                                  &hash::Precision::Medium,
                                  &hash::HashType::AHash,
                                  &self.cache)
    }

    pub fn get_dhash(&self, path: &Path) -> u64 {
        hash::get_perceptual_hash(&path,
                                  &hash::Precision::Medium,
                                  &hash::HashType::DHash,
                                  &self.cache)
    }

    pub fn get_phash(&self, path: &Path) -> u64 {
        hash::get_perceptual_hash(&path,
                                  &hash::Precision::Medium,
                                  &hash::HashType::PHash,
                                  &self.cache)
    }
}

/**
 * Get the Hamming Distance between two hashes.
 * Represents the absolute difference between two numbers.
 */
pub fn get_hamming_distance(hash1: u64, hash2: u64) -> u64 {
    hash::calculate_hamming_distance(hash1, hash2)
}

// External proxies for the get_*hash methods //

#[no_mangle]
pub extern "C" fn ext_init(cache_path_char: *const libc::c_char) -> *const libc::c_void {
    unsafe {
        let path_cstr = CStr::from_ptr(cache_path_char);
        let path_str = match path_cstr.to_str() {
            Ok(path) => Some(path),
            Err(_) => None,
        };
        // println!("Created new lib, with cache at {}", path_str.unwrap());
        let lib = Box::new(PIHash::new(path_str));
        let ptr = Box::into_raw(lib) as *mut libc::c_void;
        ptr
    }
}

#[no_mangle]
pub extern "C" fn ext_free(raw_lib: *const libc::c_void) {
    unsafe {
        drop(Box::from_raw(raw_lib as *mut PIHash));
    }
}

#[no_mangle]
pub extern "C" fn ext_get_ahash(lib: &PIHash, path_char: *const libc::c_char) -> libc::uint64_t {
    unsafe {
        let path_str = CStr::from_ptr(path_char);
        let image_path = match path_str.to_str() {
            Ok(result) => result,
            Err(e) => {
                println!("Error: {}. Unable to parse '{}'",
                         e,
                         to_hex_string(path_str.to_bytes()));
                panic!("Unable to parse path")
            }
        };
        let path = Path::new(&image_path);
        lib.get_ahash(&path)
    }
}

#[no_mangle]
pub extern "C" fn ext_get_dhash(lib: &PIHash, path_char: *const libc::c_char) -> libc::uint64_t {
    unsafe {
        let path_str = CStr::from_ptr(path_char);
        let image_path = match path_str.to_str() {
            Ok(result) => result,
            Err(e) => {
                println!("Error: {}. Unable to parse '{}'",
                         e,
                         to_hex_string(path_str.to_bytes()));
                panic!("Unable to parse path")
            }
        };
        let path = Path::new(&image_path);
        lib.get_dhash(&path)
    }
}

#[no_mangle]
pub extern "C" fn ext_get_phash(lib: &PIHash, path_char: *const libc::c_char) -> libc::uint64_t {
    unsafe {
        let path_str = CStr::from_ptr(path_char);
        let image_path = match path_str.to_str() {
            Ok(result) => result,
            Err(e) => {
                println!("Error: {}. Unable to parse '{}'",
                         e,
                         to_hex_string(path_str.to_bytes()));
                panic!("Unable to parse path")
            }
        };
        let path = Path::new(&image_path);
        lib.get_phash(&path)
    }
}

fn to_hex_string(bytes: &[u8]) -> String {
    println!("length: {}", bytes.len());
    let mut strs: Vec<String> = Vec::new();
    for byte in bytes {
        // println!("{:02x}", byte);
        strs.push(format!("{:02x}", byte));
    }
    strs.join("\\x")
}

// Module for the tests
//
#[cfg(test)]
mod tests {

    use std::fs;
    use std::path::Path;
    use hash;
    use cache;
    use super::PIHash;
    #[cfg(feature = "bench")]
    use super::test::Bencher;

    #[test]
    fn test_can_get_test_images() {
        let paths = fs::read_dir(&Path::new("./test_images")).unwrap();
        let mut num_paths = 0;
        for path in paths {
            let orig_path = path.unwrap().path();
            let ext = Path::new(&orig_path).extension();
            match ext {
                Some(_) => {
                    if ext.unwrap() == "jpg" {
                        num_paths += 1;
                        println!("Is a image {}: {:?}", num_paths, orig_path);
                    }
                }
                _ => {
                    println!("Not an image: {:?}", orig_path);
                    continue;
                }
            }
            // println!("Name: {}", path.unwrap().path().display())
        }
        // Currently 12 images in the test images directory
        assert_eq!(num_paths, 12);
    }

    /**
     * Updated test function. Assumes 3 images to a set and no hamming distances.
     * We don't need to confirm that the hamming distance calculation works in these tests.
     */
    fn test_imageset_hash(hash_type: hash::HashType,
                          hash_precision: hash::Precision,
                          max_hamming_distance: u64,
                          image_paths: [&Path; 3],
                          image_hashes: [u64; 3],
                          lib: &PIHash) {
        let mut hashes: [u64; 3] = [0; 3];
        for index in 0..image_paths.len() {
            let image_path = image_paths[index];
            let calculated_hash = lib.get_perceptual_hash(&image_path, &hash_precision, &hash_type);
            println!("[{}] Image hashes for [{}] expected: [{}] actual: [{}]",
                     hash_type,
                     image_path.to_str().unwrap(),
                     image_hashes[index],
                     calculated_hash);
            assert_eq!(calculated_hash, image_hashes[index]);
            hashes[index] = calculated_hash;
        }

        for index in 0..hashes.len() {
            for index2 in 0..hashes.len() {
                if index == index2 {
                    continue;
                } else {
                    let distance = hash::calculate_hamming_distance(hashes[index], hashes[index2]);
                    println!("Hashes [{}] and [{}] have a hamming distance of [{}] of a max allowed distance of [{}]",
                             hashes[index],
                             hashes[index2],
                             distance,
                             max_hamming_distance);
                    assert!(distance <= max_hamming_distance);
                }
            }
        }
    }

    #[test]
    fn test_confirm_ahash_results() {
        // Prep_library
        let lib = PIHash::new(Some(cache::DEFAULT_CACHE_DIR));

        // Sample_01 tests
        let sample_01_images: [&Path; 3] = [&Path::new("./test_images/sample_01_large.jpg"),
                                            &Path::new("./test_images/sample_01_medium.jpg"),
                                            &Path::new("./test_images/sample_01_small.jpg")];
        let sample_01_hashes: [u64; 3] = [7065306774709811078, 7065306774709811078, 7065306774172940166];
        test_imageset_hash(hash::HashType::AHash,
                           hash::Precision::Medium,
                           1u64,
                           sample_01_images,
                           sample_01_hashes,
                           &lib);

        // Sample_02 tests
        let sample_02_images: [&Path; 3] = [&Path::new("./test_images/sample_02_large.jpg"),
                                            &Path::new("./test_images/sample_02_medium.jpg"),
                                            &Path::new("./test_images/sample_02_small.jpg")];
        let sample_02_hashes: [u64; 3] = [18446744068986765312, 18446744069246812160, 18446744073541779456];
        test_imageset_hash(hash::HashType::AHash,
                           hash::Precision::Medium,
                           3u64,
                           sample_02_images,
                           sample_02_hashes,
                           &lib);

        // Sample_03 tests
        let sample_03_images: [&Path; 3] = [&Path::new("./test_images/sample_03_large.jpg"),
                                            &Path::new("./test_images/sample_03_medium.jpg"),
                                            &Path::new("./test_images/sample_03_small.jpg")];
        let sample_03_hashes: [u64; 3] = [108649334536274430, 126663733045756414, 108649334536274430];
        test_imageset_hash(hash::HashType::AHash,
                           hash::Precision::Medium,
                           1u64,
                           sample_03_images,
                           sample_03_hashes,
                           &lib);

        // Sample_04 tests
        let sample_04_images: [&Path; 3] = [&Path::new("./test_images/sample_04_large.jpg"),
                                            &Path::new("./test_images/sample_04_medium.jpg"),
                                            &Path::new("./test_images/sample_04_small.jpg")];
        let sample_04_hashes: [u64; 3] = [18446460933225054208, 18446460933225054208, 18446460933225054208];
        test_imageset_hash(hash::HashType::AHash,
                           hash::Precision::Medium,
                           0u64,
                           sample_04_images,
                           sample_04_hashes,
                           &lib);

        // Clean_Cache
        // super::teardown();
    }

    #[test]
    fn test_confirm_dhash_results() {
        // Prep_library
        let lib = PIHash::new(Some(cache::DEFAULT_CACHE_DIR));

        // Sample_01 tests
        let sample_01_images: [&Path; 3] = [&Path::new("./test_images/sample_01_large.jpg"),
                                            &Path::new("./test_images/sample_01_medium.jpg"),
                                            &Path::new("./test_images/sample_01_small.jpg")];
        let sample_01_hashes: [u64; 3] = [18131474507607572478, 18131474507607572478, 18131474507607572478];
        test_imageset_hash(hash::HashType::DHash,
                           hash::Precision::Medium,
                           0u64,
                           sample_01_images,
                           sample_01_hashes,
                           &lib);

        // Sample_02 tests
        let sample_02_images: [&Path; 3] = [&Path::new("./test_images/sample_02_large.jpg"),
                                            &Path::new("./test_images/sample_02_medium.jpg"),
                                            &Path::new("./test_images/sample_02_small.jpg")];
        let sample_02_hashes: [u64; 3] = [10088065226894213121, 10088065226894213121, 10088065226894213121];
        test_imageset_hash(hash::HashType::DHash,
                           hash::Precision::Medium,
                           0u64,
                           sample_02_images,
                           sample_02_hashes,
                           &lib);

        // Sample_03 tests
        let sample_03_images: [&Path; 3] = [&Path::new("./test_images/sample_03_large.jpg"),
                                            &Path::new("./test_images/sample_03_medium.jpg"),
                                            &Path::new("./test_images/sample_03_small.jpg")];
        let sample_03_hashes: [u64; 3] = [144115181601817086, 144115181601817086, 144115181601817086];
        test_imageset_hash(hash::HashType::DHash,
                           hash::Precision::Medium,
                           0u64,
                           sample_03_images,
                           sample_03_hashes,
                           &lib);

        // Sample_04 tests
        let sample_04_images: [&Path; 3] = [&Path::new("./test_images/sample_04_large.jpg"),
                                            &Path::new("./test_images/sample_04_medium.jpg"),
                                            &Path::new("./test_images/sample_04_small.jpg")];
        let sample_04_hashes: [u64; 3] = [18374262326015557633, 18374262326015557633, 18374262326283993089];
        test_imageset_hash(hash::HashType::DHash,
                           hash::Precision::Medium,
                           1u64,
                           sample_04_images,
                           sample_04_hashes,
                           &lib);

        // Clean_Cache
        // super::teardown();
    }

    #[test]
    fn test_confirm_phash_results() {
        // Prep_library
        let lib = PIHash::new(Some(cache::DEFAULT_CACHE_DIR));

        // Sample_01 tests
        let sample_01_images: [&Path; 3] = [&Path::new("./test_images/sample_01_large.jpg"),
                                            &Path::new("./test_images/sample_01_medium.jpg"),
                                            &Path::new("./test_images/sample_01_small.jpg")];
        let sample_01_hashes: [u64; 3] = [72410537899606272, 72410537899606272, 72410537899606400];
        test_imageset_hash(hash::HashType::PHash,
                           hash::Precision::Medium,
                           1u64,
                           sample_01_images,
                           sample_01_hashes,
                           &lib);

        // Sample_02 tests
        let sample_02_images: [&Path; 3] = [&Path::new("./test_images/sample_02_large.jpg"),
                                            &Path::new("./test_images/sample_02_medium.jpg"),
                                            &Path::new("./test_images/sample_02_small.jpg")];
        let sample_02_hashes: [u64; 3] = [5620563253458370560, 5620562703702556672, 5620562703702556672];
        test_imageset_hash(hash::HashType::PHash,
                           hash::Precision::Medium,
                           1u64,
                           sample_02_images,
                           sample_02_hashes,
                           &lib);

        // Sample_03 tests
        let sample_03_images: [&Path; 3] = [&Path::new("./test_images/sample_03_large.jpg"),
                                            &Path::new("./test_images/sample_03_medium.jpg"),
                                            &Path::new("./test_images/sample_03_small.jpg")];
        let sample_03_hashes: [u64; 3] = [6926536226895822848, 6926536226895822848,  6926536226895822848];
        test_imageset_hash(hash::HashType::PHash,
                           hash::Precision::Medium,
                           0u64,
                           sample_03_images,
                           sample_03_hashes,
                           &lib);

        // Sample_04 tests
        let sample_04_images: [&Path; 3] = [&Path::new("./test_images/sample_04_large.jpg"),
                                            &Path::new("./test_images/sample_04_medium.jpg"),
                                            &Path::new("./test_images/sample_04_small.jpg")];
        let sample_04_hashes: [u64; 3] = [11430286023502856200, 10997940459275288576, 11142055647351144448];
        test_imageset_hash(hash::HashType::PHash,
                           hash::Precision::Medium,
                           3u64,
                           sample_04_images,
                           sample_04_hashes,
                           &lib);

        // Clean_Cache
        // super::teardown();
    }

    #[cfg(feature = "bench")]
    #[bench]
    fn bench_with_cache(bench: &mut Bencher) -> () {
        // Prep_library
        let lib = PIHash::new(Some(cache::DEFAULT_CACHE_DIR));

        // Setup the caches to make sure we're good to properly bench
        // All phashes so that the matricies are pulled from cache as well
        lib.get_perceptual_hash(&Path::new("./test_images/sample_01_large.jpg"),
                                &hash::Precision::Medium,
                                &hash::HashType::PHash);

        bench.iter(|| {
                       // Sample_01 Bench
                       lib.get_perceptual_hash(&Path::new("./test_images/sample_01_large.jpg"),
                                               &hash::Precision::Medium,
                                               &hash::HashType::PHash);
                   })
    }

    #[cfg(feature = "bench")]
    #[bench]
    fn bench_without_cache(bench: &mut Bencher) -> () {
        // Prep_library
        let lib = PIHash::new(None);

        bench.iter(|| {
                       // Sample_01 Bench
                       lib.get_perceptual_hash(&Path::new("./test_images/sample_01_large.jpg"),
                                               &hash::Precision::Medium,
                                               &hash::HashType::PHash);
                   })
    }
}
