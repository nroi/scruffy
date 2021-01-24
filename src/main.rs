extern crate libc;

use std::ffi::CString;
use std::os::raw::c_char;
use std::fs;
use std::env;
use regex::Regex;
use std::cmp::Ordering;
use std::cmp::Ordering::{Less, Greater, Equal};
use itertools::Itertools;
use std::path::PathBuf;

extern "C" {
    fn alpm_pkg_vercmp(a: *const c_char, b: *const c_char) -> i32;
}

fn vercmp(a: &str, b: &str) -> Ordering {
    let s1 = CString::new(a).unwrap();
    let s2 = CString::new(b).unwrap();
    unsafe {
        match alpm_pkg_vercmp(s1.as_ptr(), s2.as_ptr()) {
            -1 => Less,
            0 => Equal,
            1 => Greater,
            e => panic!("Unexpected comparison result: {}", e)
        }
    }
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
struct PkgVersion {
    version: String,
}

impl Ord for PkgVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        vercmp(&self.version, &other.version)
    }
}

impl PartialOrd for PkgVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
struct PkgFile {
    pkg_info: PkgInfo,
    path_buf: PathBuf,
}

impl Ord for PkgFile {
    fn cmp(&self, other: &Self) -> Ordering {
        self.pkg_info.cmp(&other.pkg_info)
    }
}

impl PartialOrd for PkgFile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Hash, Clone, PartialEq, Eq, Ord, PartialOrd)]
struct PkgInfo  {
    name: String,
    version: PkgVersion,
}

fn main() {

    let args: Vec<String> = env::args().collect();
    let path = &args[1];
    let keep = args[2].parse::<usize>().unwrap();

    let re = Regex::new(r"^(?P<name>.*)-(?P<version>[^-]*-[^-]*)-[^-]*$").unwrap();
    let paths = fs::read_dir(path).unwrap();
    let pkg_infos: Vec<PkgFile> = paths.filter_map(|path| {
        let pp = path.unwrap();
        let filename = pp.file_name().to_str().unwrap().to_owned();
        let path = pp.path();
        match re.captures(&filename) {
            None => None,
            Some(c) => {
                let name = c.name("name").unwrap().as_str().to_owned();
                let version = c.name("version").unwrap().as_str().to_owned();
                let pkg_info = PkgInfo {
                    name,
                    version: PkgVersion {
                        version
                    }
                };
                let pkg_file = PkgFile {
                    pkg_info,
                    path_buf: path,
                };
                Some(pkg_file)
            }
        }
    }).sorted_unstable().collect();

    let mut file_size = 0;
    let mut num_candidates = 0;
    for (_, pkg_files) in &pkg_infos.iter().group_by(|p| &p.pkg_info.name) {
        let pkg_files = pkg_files.collect_vec();
        for pkg_file in pkg_files.iter().rev().skip(keep) {
            println!("{}", pkg_file.path_buf.file_name().unwrap().to_str().unwrap());
            file_size += pkg_file.path_buf.metadata().expect("Unable to fetch file metadata").len();
            num_candidates += 1;
        }
    }

    println!("num candidates: {}", num_candidates);
    println!("file size: {}", size_to_human_readable(file_size));
}

pub fn size_to_human_readable(size_in_bytes: u64) -> String {
    let exponent = ((size_in_bytes as f64).log2() / 10.0) as u32;
    let (unit, too_large) = match exponent {
        0 => ("B", false),
        1 => ("KiB", false),
        2 => ("MiB", false),
        3 => ("GiB", false),
        4 => ("TiB", false),
        5 => ("PiB", false),
        6 => ("EiB", false),
        7 => ("ZiB", false),
        8 => ("YiB", false),
        _ => ("B", true),
    };
    if too_large {
        format!("{}", size_in_bytes)
    } else {
        let quantity = (size_in_bytes as f64) / ((1024u64).pow(exponent) as f64);
        format!("{:.2} {}", quantity, unit)
    }
}
