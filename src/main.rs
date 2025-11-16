use clap::{
    Arg, ColorChoice, Command,
    builder::styling::{AnsiColor, Effects, Styles},
};
use git2::{Repository, string_array::StringArray};
use regex::Regex;
use semver::Version;
use std::{collections::BTreeSet, env, process};

// matches a SemVer optionally prefixed by v/V and captures the normalized version string
const SEMVER_RX: &str = r"[vV]?(?P<version>(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?)";

fn main() {
    let styles = Styles::styled()
        .header(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .usage(AnsiColor::Green.on_default() | Effects::BOLD)
        .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Green.on_default());

    // cli options default to patch
    let matches = Command::new("gbump")
        .version(env!("CARGO_PKG_VERSION"))
        .color(ColorChoice::Auto)
        .styles(styles)
        .arg(
            Arg::new("bump")
                .default_value("patch")
                .value_parser(["major", "minor", "patch"])
                .num_args(1),
        )
        .arg(
            Arg::new("quiet")
                .long("quiet")
                .short('q')
                .help("Prints only the next SemVer not the current one")
                .num_args(0),
        )
        .arg(
            Arg::new("tag")
                .long("tag")
                .short('t')
                .help("Create a semver git tag")
                .num_args(0),
        )
        .get_matches();

    // check if we are in a git repository
    let repo = Repository::discover(".").unwrap_or_else(|_| fatal("Not in a git repository"));

    // find maximum/latest semver
    let version = tags(&repo).map_or_else(
        |_| fatal("Could not get tags from repo: git tag -l"),
        |tags| semver(&tags),
    );

    // prepare the output
    let mut semver = String::new();

    if !matches.get_flag("quiet") {
        semver.push_str(format!("{version} --> ").as_str());
    }

    let bump = bump(
        matches
            .get_one::<String>("bump")
            .expect("clap default ensures value"),
        &version,
    )
    .expect("value parser restricts bump choices");
    let bump_str = bump.to_string();

    semver.push_str(&bump_str);
    println!("{semver}");

    if matches.get_flag("tag") {
        tag(&repo, bump_str.as_str(), bump_str.as_str()).map_or_else(
            |e| fatal(format!("Could not create tag: {e}")),
            |n| println!("Tag: {bump_str} created: {n}"),
        );
    }
}

// create a tag: git tag -a bump -m bump
fn tag(repo: &Repository, tag: &str, message: &str) -> Result<git2::Oid, git2::Error> {
    let obj = repo.revparse_single("HEAD")?;
    let sig = repo.signature()?;
    repo.tag(tag, &obj, &sig, message, false)
}

// return bumped version or None if the requested bump is invalid
fn bump(target: &str, version: &Version) -> Option<Version> {
    match target {
        "major" => Some(Version::new(version.major + 1, 0, 0)),
        "minor" => Some(Version::new(version.major, version.minor + 1, 0)),
        "patch" => Some(Version::new(
            version.major,
            version.minor,
            version.patch + 1,
        )),
        _ => None,
    }
}

// return tags found in the repository
fn tags(repo: &Repository) -> Result<BTreeSet<String>, git2::Error> {
    let mut tags = BTreeSet::new();
    for tag in tag_names(repo)?.iter().flatten() {
        tags.insert(tag.to_string());
    }
    Ok(tags)
}

fn tag_names(repo: &Repository) -> Result<StringArray, git2::Error> {
    if env::var_os("GBUMP_FORCE_TAG_FAILURE").is_some() {
        Err(git2::Error::from_str("forced tags failure"))
    } else {
        repo.tag_names(None)
    }
}

// return highest SemVer taking prerelease/build metadata into account
fn semver(tags: &BTreeSet<String>) -> Version {
    let re = Regex::new(SEMVER_RX).unwrap();
    let mut best: Option<Version> = None;
    for tag in tags {
        for caps in re.captures_iter(tag) {
            if let Ok(version) = Version::parse(
                caps.name("version")
                    .expect("regex ensures version capture")
                    .as_str(),
            ) && (best.as_ref().is_none() || version > *best.as_ref().unwrap())
            {
                best = Some(version);
            }
        }
    }
    best.unwrap_or_else(|| Version::new(0, 0, 0))
}

fn fatal(message: impl AsRef<str>) -> ! {
    eprintln!("{}", message.as_ref());
    process::exit(1);
}

#[cfg(test)]
mod tests {
    use crate::{bump, semver, tag, tags};
    use git2::{Repository, Signature};
    use semver::Version;
    use std::{collections::BTreeSet, path::Path};
    use tempfile::TempDir;

    #[test]
    fn test_semver_major() {
        let mut tags = BTreeSet::<String>::new();
        tags.insert("0.1.2".to_string());
        tags.insert("3.7.0".to_string());
        tags.insert("1.17.1".to_string());
        tags.insert("2.7.2".to_string());
        tags.insert("0.24.0".to_string());
        let version = semver(&tags);
        assert_eq!(version.major, 3);
        assert_eq!(version.minor, 7);
        assert_eq!(version.patch, 0);
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
        let version = semver(&tags);
        assert_eq!(version.major, 0);
        assert_eq!(version.minor, 24);
        assert_eq!(version.patch, 0);
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
        let version = semver(&tags);
        assert_eq!(version.major, 0);
        assert_eq!(version.minor, 99);
        assert_eq!(version.patch, 100);
    }

    #[test]
    // https://regex101.com/r/ahzkLW/1/
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
        let version = semver(&tags);
        assert_eq!(version.major, 12);
        assert_eq!(version.minor, 1);
        assert_eq!(version.patch, 0);
    }

    #[test]
    fn test_bump() {
        let base = Version::new(0, 0, 0);
        assert_eq!(bump("patch", &base).unwrap().to_string(), "0.0.1");
        assert_eq!(bump("minor", &base).unwrap().to_string(), "0.1.0");
        assert_eq!(bump("major", &base).unwrap().to_string(), "1.0.0");

        let mixed = Version::new(1, 2, 3);
        assert_eq!(bump("patch", &mixed).unwrap().to_string(), "1.2.4");
        assert_eq!(bump("minor", &mixed).unwrap().to_string(), "1.3.0");
        assert_eq!(bump("major", &mixed).unwrap().to_string(), "2.0.0");
    }

    #[test]
    fn test_bump_invalid() {
        assert!(bump("foo", &Version::new(0, 0, 0)).is_none());
    }

    #[test]
    fn test_semver_preserves_metadata() {
        let mut tags = BTreeSet::<String>::new();
        tags.insert("release-1.2.3-beta.11+build.5".to_string());
        let version = semver(&tags);
        assert_eq!(version.to_string(), "1.2.3-beta.11+build.5");
    }

    #[test]
    fn test_semver_prerelease_ordering() {
        let mut tags = BTreeSet::<String>::new();
        tags.insert("v1.0.0-alpha".to_string());
        tags.insert("1.0.0-alpha.1".to_string());
        tags.insert("1.0.0-alpha.beta".to_string());
        tags.insert("1.0.0-beta".to_string());
        tags.insert("1.0.0-beta.2".to_string());
        tags.insert("1.0.0-beta.11".to_string());
        tags.insert("1.0.0-rc.1".to_string());
        let version = semver(&tags);
        assert_eq!(version.to_string(), "1.0.0-rc.1");
    }

    #[test]
    fn test_semver_ignores_invalid_entry() {
        let mut tags = BTreeSet::<String>::new();
        tags.insert("99999999999999999999999999999999999.0.0".to_string());
        tags.insert("0.0.1".to_string());
        let version = semver(&tags);
        assert_eq!(version.to_string(), "0.0.1");
    }

    #[test]
    fn test_semver_defaults_to_zero() {
        let tags = BTreeSet::<String>::new();
        let version = semver(&tags);
        assert_eq!(version.to_string(), "0.0.0");
    }

    #[test]
    fn test_tags_function_reads_git_tags() {
        let (_tmp, repo) = init_repo();
        tag(&repo, "1.0.0", "1.0.0").unwrap();
        tag(&repo, "v1.1.0", "v1.1.0").unwrap();
        let names = tags(&repo).unwrap();
        assert!(names.contains("1.0.0"));
        assert!(names.contains("v1.1.0"));
    }

    #[test]
    fn test_tag_function_creates_tag() {
        let (_tmp, repo) = init_repo();
        let oid = tag(&repo, "2.0.0", "release").unwrap();
        let names = repo.tag_names(None).unwrap();
        assert!(names.iter().flatten().any(|name| name == "2.0.0"));
        let tag_ref = repo.find_reference("refs/tags/2.0.0").unwrap();
        assert_eq!(tag_ref.target().unwrap(), oid);
    }

    #[cfg(unix)]
    #[test]
    fn test_tags_skips_invalid_utf8_entries() {
        use std::os::unix::ffi::OsStringExt;

        let (_tmp, repo) = init_repo();
        tag(&repo, "1.0.0", "1.0.0").unwrap();
        let bad_name = std::ffi::OsString::from_vec(vec![b'b', 0xFF, b'a', b'd']);
        let bad_path = repo.path().join("refs").join("tags").join(&bad_name);
        let head = repo.head().unwrap().target().unwrap();
        std::fs::write(bad_path, format!("{head}\n")).unwrap();
        let names = tags(&repo).unwrap();
        assert!(names.contains("1.0.0"));
    }

    fn init_repo() -> (TempDir, Repository) {
        let tmp = TempDir::new().unwrap();
        let repo = Repository::init(tmp.path()).unwrap();
        {
            let mut config = repo.config().unwrap();
            config.set_str("user.name", "Tester").unwrap();
            config.set_str("user.email", "tester@example.com").unwrap();
        }
        std::fs::write(tmp.path().join("README"), "test").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("README")).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        {
            let tree = repo.find_tree(tree_id).unwrap();
            let sig = Signature::now("Tester", "tester@example.com").unwrap();
            repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
                .unwrap();
        }
        (tmp, repo)
    }
}
