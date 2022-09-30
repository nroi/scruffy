extern crate libc;

use std::ffi::CString;
use std::os::raw::c_char;
use std::fs;
use regex::Regex;
use std::cmp::Ordering;
use std::cmp::Ordering::{Less, Greater, Equal};
use itertools::Itertools;
use std::path::PathBuf;
use walkdir::WalkDir;
use clap::Clap;

extern "C" {
    fn alpm_pkg_vercmp(a: *const c_char, b: *const c_char) -> i32;
}

#[derive(Clap)]
#[clap()]
struct Opts {
    /// The cache directory. This directory will be searched recursively.
    #[clap(short, long)]
    cachedir: String,
    /// verbose mode
    #[clap(short, long)]
    verbose: bool,
    /// Perform a dry run, only finding candidate packages.
    #[clap(short, long)]
    dryrun: bool,
    /// Specify how many versions of each package are kept in the cache directory.
    #[clap(short, long, default_value = "3")]
    keep: usize,
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
    let opts: Opts = Opts::parse();
    let cache_dir = format!("{}/", opts.cachedir);
    let pkg_files = pkg_files(&cache_dir);

    let mut file_size = 0;
    let mut num_candidates = 0;
    for (_, pkg_files) in &pkg_files.iter().group_by(|p| &p.pkg_info.name) {
        let pkg_files = pkg_files.collect_vec();
        for pkg_file in pkg_files.iter().rev().skip(opts.keep) {
            if opts.verbose {
                let deletion_candidate = pkg_file.path_buf.to_str().unwrap().strip_prefix(&cache_dir).unwrap();
                println!("{}", &deletion_candidate);
            }
            file_size += pkg_file.path_buf.metadata().expect("Unable to fetch file metadata").len();
            num_candidates += 1;
            if !opts.dryrun {
                let pkg_file_name = pkg_file.path_buf.file_name().unwrap().to_str().unwrap();
                let sig_file_name = format!("{}.sig", pkg_file_name);
                let cfs_file_name = format!(".{}.cfs", pkg_file_name);
                let sig_cfs_file_name = format!(".{}.sig.cfs", pkg_file_name);
                let sig_file = pkg_file.path_buf.with_file_name(sig_file_name);
                let cfs_file = pkg_file.path_buf.with_file_name(cfs_file_name);
                let sig_cfs_file = pkg_file.path_buf.with_file_name(sig_cfs_file_name);
                fs::remove_file(&pkg_file.path_buf).unwrap();
                // The corresponding CFS and signature files should be removed too, if they exist:
                let _ = fs::remove_file(sig_file);
                let _ = fs::remove_file(cfs_file);
                let _ = fs::remove_file(sig_cfs_file);
            }
        }
    }

    let prefix = if opts.dryrun {
        "[dry run] "
    } else {
        ""
    };
    if opts.verbose {
        println!("\n{}{} packages removed (disk space saved: {})",
                 prefix, num_candidates, size_to_human_readable(file_size));
    }
}

fn pkg_files(cache_dir: &str) -> Vec<PkgFile> {
    let re = Regex::new(r"^(?P<name>.*)-(?P<version>[^-]*-[^-]*)-[^-]*$").unwrap();
    WalkDir::new(&cache_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| {
            let ignore_file = match entry.path().extension() {
                None => false,
                Some(ext) => {
                    // Both CFS files and signature files are ignored, because this function should return
                    // only real package files.
                    ext.eq_ignore_ascii_case("cfs") || ext.eq_ignore_ascii_case("sig")
                }
            };
            !ignore_file
        })
        .filter_map(|entry| {
        let path = entry.path();
        let filename = path.file_name().unwrap().to_str().unwrap().to_owned();
        re.captures(&filename).map(|c| {
            let name = c.name("name").unwrap().as_str().to_owned();
            let version = c.name("version").unwrap().as_str().to_owned();
            let pkg_info = PkgInfo {
                name,
                version: PkgVersion {
                    version
                }
            };
            PkgFile {
                pkg_info,
                path_buf: path.to_path_buf(),
            }
        })
    }).sorted_unstable().collect()
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

fn size_to_human_readable(size_in_bytes: u64) -> String {
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

