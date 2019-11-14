use clap::{App, Arg};
use git2::Repository;
use regex::Regex;
use std::{collections::BTreeSet, env, process};

const SEMVER_RX: &str = r"(?P<major>0|[1-9]\d*)\.(?P<minor>0|[1-9]\d*)\.(?P<patch>0|[1-9]\d*)";

fn main() {
    // cli options default to patch
    let matches = App::new("bump")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::with_name("version")
                .required(true)
                .takes_value(false)
                .default_value("patch")
                .possible_value("major")
                .possible_value("minor")
                .possible_value("patch"),
        )
        .get_matches();

    let repo = match env::current_dir() {
        Ok(path) => match Repository::discover(path) {
            Ok(repo) => repo,
            _ => {
                eprintln!("not in a git repository");
                process::exit(1);
            }
        },
        _ => {
            eprintln!("could not get current_dir");
            process::exit(1);
        }
    };

    let tags = get_tags(repo).unwrap();
    let (major, minor, patch) = semver(tags);
    let current_version = format!("{}.{}.{}", major, minor, patch);
    match matches.value_of("version").unwrap() {
        "major" => println!("{} --> {}.{}.{}", current_version, major + 1, minor, patch),
        "minor" => println!("{} --> {}.{}.{}", current_version, major, minor + 1, patch),
        "patch" => println!("{} --> {}.{}.{}", current_version, major, minor, patch + 1),
        _ => (),
    }
}

fn get_tags(repo: Repository) -> Result<BTreeSet<String>, git2::Error> {
    let mut tags = BTreeSet::new();
    for name in repo.tag_names(None)?.iter() {
        if let Some(tag) = name {
            tags.insert(tag.to_string());
        }
    }
    Ok(tags)
}

fn semver(tags: BTreeSet<String>) -> (usize, usize, usize) {
    let re = Regex::new(SEMVER_RX).unwrap();
    let (mut major, mut minor, mut patch) = (0, 0, 0);
    for tag in &tags {
        if let Some(caps) = re.captures(tag) {
            let x = caps["major"].parse::<usize>().unwrap();
            let y = caps["minor"].parse::<usize>().unwrap();
            let z = caps["patch"].parse::<usize>().unwrap();
            if x > major {
                major = x;
                minor = y;
                patch = z;
            } else if x == major && y > minor {
                minor = y;
                patch = z;
            } else if x == major && y == minor && z > patch {
                patch = z;
            }
        }
    }
    (major, minor, patch)
}

#[cfg(test)]
mod tests {
    use crate::semver;
    use std::collections::BTreeSet;

    #[test]
    fn test_semver_major() {
        let mut tags = BTreeSet::<String>::new();
        tags.insert("0.1.2".to_string());
        tags.insert("3.7.0".to_string());
        tags.insert("1.17.1".to_string());
        tags.insert("2.7.2".to_string());
        tags.insert("0.24.0".to_string());
        let (major, minor, patch) = semver(tags);
        println!("{}.{}.{}", major, minor, patch);
        assert_eq!(major, 3);
        assert_eq!(minor, 7);
        assert_eq!(patch, 0);
    }

    #[test]
    fn test_semver_minor() {
        let mut tags = BTreeSet::<String>::new();
        tags.insert("0.1.2".to_string());
        tags.insert("0.7.0".to_string());
        tags.insert("0.7.1".to_string());
        tags.insert("0.7.2".to_string());
        tags.insert("0.2.2".to_string());
        tags.insert("0.9.0".to_string());
        tags.insert("0.8.3".to_string());
        tags.insert("0.23.0".to_string());
        tags.insert("0.24.0".to_string());
        let (major, minor, patch) = semver(tags);
        println!("{}.{}.{}", major, minor, patch);
        assert_eq!(major, 0);
        assert_eq!(minor, 24);
        assert_eq!(patch, 0);
    }

    #[test]
    fn test_semver_patch() {
        let mut tags = BTreeSet::<String>::new();
        tags.insert("0.1.2".to_string());
        tags.insert("0.7.0".to_string());
        tags.insert("0.7.1".to_string());
        tags.insert("0.7.2".to_string());
        tags.insert("0.2.2".to_string());
        tags.insert("0.9.0".to_string());
        tags.insert("0.8.3".to_string());
        tags.insert("0.23.0".to_string());
        tags.insert("0.24.0".to_string());
        tags.insert("0.99.100".to_string());
        let (major, minor, patch) = semver(tags);
        println!("{}.{}.{}", major, minor, patch);
        assert_eq!(major, 0);
        assert_eq!(minor, 99);
        assert_eq!(patch, 100);
    }

    #[test]
    fn test_semver_regex() {
        let mut tags = BTreeSet::<String>::new();
        tags.insert("1.2.3".to_string());
        tags.insert("10.20.30".to_string());
        tags.insert("1.1.2-prerelease+meta".to_string());
        tags.insert("1.1.2+meta".to_string());
        tags.insert("1.1.2+meta-valid".to_string());
        tags.insert("1.0.0-alpha".to_string());
        tags.insert("1.0.0-beta".to_string());
        tags.insert("1.0.0-alpha.beta".to_string());
        tags.insert("1.0.0-alpha.beta.1".to_string());
        tags.insert("1.0.0-alpha.1".to_string());
        tags.insert("1.0.0-alpha0.valid".to_string());
        tags.insert("1.0.0-alpha.0valid".to_string());
        tags.insert("1.0.0-alpha-a.b-c-somethinglong+build.1-aef.1-its-okay".to_string());
        tags.insert("1.0.0-rc.1+build.1".to_string());
        tags.insert("2.0.0-rc.1+build.123".to_string());
        tags.insert("1.2.3-beta".to_string());
        tags.insert("10.2.3-DEV-SNAPSHOT".to_string());
        tags.insert("1.2.3-SNAPSHOT-123".to_string());
        tags.insert("1.0.0".to_string());
        tags.insert("2.0.0".to_string());
        tags.insert("1.1.7".to_string());
        tags.insert("2.0.0+build.1848".to_string());
        tags.insert("2.0.1-alpha.1227".to_string());
        tags.insert("1.0.0-alpha+beta".to_string());
        tags.insert("1.2.3----RC-SNAPSHOT.12.9.1--.12+788".to_string());
        tags.insert("1.2.3----R-S.12.9.1--.12+meta".to_string());
        tags.insert("1.2.3----RC-SNAPSHOT.12.9.1--.12".to_string());
        tags.insert("1.0.0+0.build.1-rc.10000aaa-kk-0.1".to_string());
        tags.insert("0.999999999999999999.99999999999999999".to_string());
        tags.insert("1.0.0-0A.is.legal".to_string());
        tags.insert("v1.1.1".to_string());
        tags.insert("1.1.1".to_string());
        tags.insert("0.0.0".to_string());
        tags.insert("v0.0.3".to_string());
        tags.insert("0.0.0".to_string());
        tags.insert("1.1.1  1.1".to_string());
        tags.insert("12.1.0---FreeBSD.12.1-RELEASE".to_string());
        let (major, minor, patch) = semver(tags);
        println!("{}.{}.{}", major, minor, patch);
        assert_eq!(major, 12);
        assert_eq!(minor, 1);
        assert_eq!(patch, 0);
    }
}
