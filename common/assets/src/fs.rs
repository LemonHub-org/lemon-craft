use std::{fs, io};

use assets_manager::{
    BoxedError,
    hot_reloading::{EventSender, FsWatcherBuilder},
    source::{DirEntry, FileContent, FileSystem as RawFs, Source},
};
use hashbrown::HashSet;

/// Loads assets from the default path, optionally overlaying content packs
/// (in priority order, later packs override earlier ones) and the legacy
/// `VELOREN_ASSETS_OVERRIDE` environment variable (highest priority).
#[derive(Debug, Clone)]
pub struct FileSystem {
    default: RawFs,
    packs: Vec<RawFs>,
}

impl FileSystem {
    pub fn new() -> io::Result<Self> {
        let default = RawFs::new(&*super::ASSETS_PATH)?;
        let mut packs = Vec::new();
        if let Some(path) = std::env::var_os("VELOREN_ASSETS_OVERRIDE") {
            match RawFs::new(path) {
                Ok(fs) => packs.push(fs),
                Err(err) => tracing::error!("Error setting override assets directory: {}", err),
            }
        }

        let canary = fs::read_to_string(super::ASSETS_PATH.join("common").join("canary.canary"))
            .map_err(|e| io::Error::other(format!("failed to load canary asset: {}", e)))?;

        if !canary.starts_with("VELOREN_CANARY_MAGIC") {
            panic!("Canary asset `canary.canary` was present but did not contain the expected data. This *heavily* implies that you've not correctly set up Git LFS (Large File Storage). Visit `https://book.veloren.net/contributors/development-tools.html#git-lfs` for more information about setting up Git LFS.");
        }

        Ok(Self { default, packs })
    }
}

impl Source for FileSystem {
    fn read(&self, id: &str, ext: &str) -> io::Result<FileContent<'_>> {
        // Check packs in reverse order: the last (highest priority) pack wins.
        for dir in self.packs.iter().rev() {
            match dir.read(id, ext) {
                Ok(content) => return Ok(content),
                Err(err) => {
                    if err.kind() != io::ErrorKind::NotFound {
                        let path = dir.path_of(DirEntry::File(id, ext));
                        tracing::warn!(
                            "Error reading \"{}\": {}. Falling back to lower-priority source",
                            path.display(),
                            err
                        );
                    }
                },
            }
        }

        // If not found in any pack, try load from main asset path
        self.default.read(id, ext)
    }

    fn read_dir(&self, id: &str, f: &mut dyn FnMut(DirEntry)) -> io::Result<()> {
        // It's easy to get wrong, so here's the algorithm:
        //
        // 1) Read default assets directory first, gather directories it has.
        // 2) Read each pack directory in priority order (low to high), gathering
        //    directories *it* has.
        // 3) Call callback on each new directory (or file).
        //
        // This should route to src.read() above, which does read higher-priority
        // packs first, so even if we search for default directories first, we're
        // still overriding files proper.
        //
        // The rest is just properly routing errors.
        let mut collected = HashSet::new();

        let mut f = |dir_entry: DirEntry| {
            let cache_id = match dir_entry {
                DirEntry::File(path, ext) => (path.to_owned(), Some(ext.to_owned())),
                DirEntry::Directory(path) => (path.to_owned(), None),
            };

            // on first hit, call the callback
            if collected.insert(cache_id) {
                f(dir_entry)
            }
        };

        let mut last_err = self.default.read_dir(id, &mut f);

        for dir in &self.packs {
            let pack_res = match dir.read_dir(id, &mut f) {
                Ok(()) => Ok(()),
                Err(err) => {
                    if err.kind() != io::ErrorKind::NotFound {
                        let path = dir.path_of(DirEntry::Directory(id));
                        tracing::warn!(
                            "Error reading \"{}\": {}. Falling back to lower-priority source",
                            path.display(),
                            err
                        );
                    }
                    Err(err)
                },
            };

            // Error juggling
            match (last_err, pack_res) {
                // If failed from the start, error.
                //
                // Technically not necessary, but better be safe then sorry?
                (Err(err1), _) if err1.kind() != io::ErrorKind::NotFound => last_err = Err(err1),
                // If pack succeeded, cool, celebrate.
                (_, Ok(())) => last_err = Ok(()),
                // If pack failed, but a lower-priority source succeeded, who cares.
                //
                // We could be strict here, but overrides are brittle by design,
                // and may fail with new version, so ...
                //
                // We log the warning there, that's it.
                (Ok(()), Err(_)) => last_err = Ok(()),
                // If both failed, return last error.
                (Err(_), Err(err2)) => last_err = Err(err2),
            }
        }

        last_err
    }

    fn exists(&self, entry: DirEntry) -> bool {
        self.packs.iter().any(|dir| dir.exists(entry)) || self.default.exists(entry)
    }

    fn configure_hot_reloading(&self, events: EventSender) -> Result<(), BoxedError> {
        let mut builder = FsWatcherBuilder::new()?;

        for dir in &self.packs {
            builder.watch(dir.root().to_owned())?;
        }
        builder.watch(self.default.root().to_owned())?;

        builder.build(events);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::Path};

    pub(super) enum FsNode<'a> {
        File(&'a str, &'a str),
        Dir(&'a str, Vec<FsNode<'a>>),
    }

    impl FileSystem {
        pub(super) fn scope<R>(f: &dyn Fn(FileSystem, &Path, &Path) -> R) -> R {
            Self::scope_with_packs(&|fs, main_path, pack1_path, _pack2_path| {
                f(fs, main_path, pack1_path)
            })
        }

        /// Like `scope`, but with two pack directories (each with its own
        /// tempdir). Packs are appended in order, so the later pack takes
        /// priority over the earlier one.
        pub(super) fn scope_with_packs<R>(f: &dyn Fn(FileSystem, &Path, &Path, &Path) -> R) -> R {
            let tempdir = tempfile::tempdir().expect("failed to get tempdir");
            let default = RawFs::new(tempdir.path())
                .expect("failed to create temporary filesystem for assets");

            let tempdir_pack1 = tempfile::tempdir().expect("failed to get tempdir for packs");
            let pack1 = RawFs::new(tempdir_pack1.path())
                .expect("failed to create temporary pack filesystem");
            let tempdir_pack2 = tempfile::tempdir().expect("failed to get tempdir for packs");
            let pack2 = RawFs::new(tempdir_pack2.path())
                .expect("failed to create temporary pack filesystem");

            // NOTE: we're using closure pattern here, because otherwise
            // tempdirs would get dropped about here, and run their
            // destructors, which would remove directories.
            // Instead they will get called at the end of this function,
            // after the test closure gets called.
            let this = Self {
                default,
                packs: vec![pack1, pack2],
            };
            f(
                this,
                tempdir.path(),
                tempdir_pack1.path(),
                tempdir_pack2.path(),
            )
        }

        pub(super) fn read_to_str(&self, id: &str, ext: &str) -> String {
            std::str::from_utf8(self.read(id, ext).unwrap().as_ref())
                .unwrap()
                .to_owned()
        }

        pub(super) fn mock_file(dir: &Path, filename: &str, content: &str) {
            fs::write(dir.join(filename), content).unwrap();
        }

        pub(super) fn mock_tree(dir: &Path, tree: Vec<FsNode<'_>>) {
            fn create_mock_node(path: &Path, node: FsNode<'_>) {
                match node {
                    FsNode::File(name, content) => FileSystem::mock_file(path, name, content),
                    FsNode::Dir(name, entries) => {
                        for entry in entries {
                            fs::create_dir_all(path.join(name)).unwrap();
                            create_mock_node(&path.join(name), entry);
                        }
                    },
                }
            }

            for entry in tree {
                create_mock_node(dir, entry);
            }
        }
    }

    // -- Some basic tests for the DSL above

    #[test]
    fn test_mock_tree() {
        FileSystem::scope(&|fs, main_path, _override_path| {
            FileSystem::mock_tree(main_path, vec![FsNode::File("template.ron", "(5)")]);

            assert_eq!(fs.read_to_str("template", "ron"), "(5)");
        })
    }

    #[test]
    #[should_panic(expected = "assertion `left == right` failed")]
    fn test_mock_file_properly_fails() {
        FileSystem::scope(&|fs, main_path, _override_path| {
            FileSystem::mock_file(main_path, "template.ron", "(5)");

            assert_eq!(fs.read_to_str("template", "ron"), "(6)");
        })
    }

    // -- Now finally testing our FileSystem

    #[test]
    fn test_read_main() {
        FileSystem::scope(&|fs, main_path, _override_path| {
            FileSystem::mock_file(main_path, "template.ron", "(5)");

            assert_eq!(fs.read_to_str("template", "ron"), "(5)");
        })
    }

    #[test]
    fn test_read_override() {
        FileSystem::scope(&|fs, _main_path, override_path| {
            FileSystem::mock_file(override_path, "template.ron", "(5)");

            assert_eq!(fs.read_to_str("template", "ron"), "(5)");
        })
    }

    #[test]
    fn test_read_dir() {
        FileSystem::scope(&|fs, main_path, _override_path| {
            #[rustfmt::skip]
            FileSystem::mock_tree(main_path, vec![
                FsNode::Dir("entity", vec![
                    FsNode::File("template.ron", "(5)")
                ]),
            ]);

            assert_eq!(fs.read_to_str("entity.template", "ron"), "(5)");
        })
    }

    #[test]
    fn test_read_dirfile_override() {
        FileSystem::scope(&|fs, main_path, override_path| {
            #[rustfmt::skip]
            FileSystem::mock_tree(main_path, vec![
                FsNode::Dir("entity", vec![
                    FsNode::File("template.ron", "(5)")
                ]),
            ]);

            #[rustfmt::skip]
            FileSystem::mock_tree(override_path, vec![
                FsNode::Dir("entity", vec![
                    FsNode::File("template.ron", "(6)")
                ]),
            ]);

            assert_eq!(fs.read_to_str("entity.template", "ron"), "(6)");
        })
    }

    #[test]
    fn test_read_dirfile_override_only() {
        FileSystem::scope(&|fs, _main_path, override_path| {
            #[rustfmt::skip]
            FileSystem::mock_tree(override_path, vec![
                FsNode::Dir("entity", vec![
                    FsNode::File("template.ron", "(5)")
                ]),
            ]);

            assert_eq!(fs.read_to_str("entity.template", "ron"), "(5)");
        })
    }

    #[test]
    fn test_read_dirfile_partial_override() {
        FileSystem::scope(&|fs, main_path, override_path| {
            // creating dir with two files
            #[rustfmt::skip]
            FileSystem::mock_tree(main_path, vec![
                FsNode::Dir("entity", vec![
                    FsNode::File("template.ron", "(5)"),
                    FsNode::File("main.ron", "(7)")
                ]),
            ]);

            // overriding only template here, main is still same
            #[rustfmt::skip]
            FileSystem::mock_tree(override_path, vec![
                FsNode::Dir("entity", vec![
                    FsNode::File("template.ron", "(5)")
                ]),
            ]);

            assert_eq!(fs.read_to_str("entity.template", "ron"), "(5)");
            assert_eq!(fs.read_to_str("entity.main", "ron"), "(7)");
        })
    }

    #[test]
    // I still dont understand how this one can fails while
    // previous one doesn't, but that's why Source has two methods, I suppose.
    //
    // At the time of writing, broken implementation passed previous, but not
    // that one.
    //
    // P.s. the difference is that before we were asserting fs.read(), and this
    // time we're asserting fs.read_dir(), and apparently fs.read() works
    // independently of fs.read_dir().
    fn test_read_dir_actually() {
        FileSystem::scope(&|fs, main_path, override_path| {
            // creating dir with two files
            #[rustfmt::skip]
            FileSystem::mock_tree(main_path, vec![
                FsNode::Dir("entity", vec![
                    FsNode::File("template.ron", "(5)"),
                    FsNode::File("main.ron", "(7)")
                ]),
            ]);

            // overriding only template here, main is still same
            #[rustfmt::skip]
            FileSystem::mock_tree(override_path, vec![
                FsNode::Dir("entity", vec![
                    FsNode::File("template.ron", "(6)"),
                    FsNode::File("fun.ron", "(5)")
                ]),
            ]);

            let mut files: Vec<String> = vec![];
            let _ = fs.read_dir("entity", &mut |e: DirEntry| match e {
                DirEntry::File(path, ext) => files.push(format!("{path}.{ext}")),
                DirEntry::Directory(path) => files.push(format!("{path}/")),
            });
            files.sort();
            assert_eq!(files, vec![
                // override only
                "entity.fun.ron".to_owned(),
                // main only
                "entity.main.ron".to_owned(),
                // shared and overriden
                "entity.template.ron".to_owned(),
            ]);
        })
    }

    #[test]
    fn test_read_dir_notfound() {
        FileSystem::scope(&|fs, main_path, override_path| {
            // creating dir with two files
            #[rustfmt::skip]
            FileSystem::mock_tree(main_path, vec![
                FsNode::Dir("entity", vec![FsNode::File(
                    "template.ron",
                    "(5)",
                )])
            ]);

            // creating dir with two files
            #[rustfmt::skip]
            FileSystem::mock_tree(override_path, vec![
                FsNode::Dir("entity", vec![FsNode::File(
                    "template.ron",
                    "(5)",
                )])
            ]);

            // Reading non-existent file should report the error and a path
            //
            // NOTE: basically a guard for potential assets_manager regressions
            // since uh, things accidentally happened in the past.
            let res = fs.read("loadout.template", "ron");
            assert_eq!(res.as_ref().unwrap_err().kind(), io::ErrorKind::NotFound);
            let msg = format!("{:#?}", res.unwrap_err());
            // The error path uses the platform separator (`/` on Unix, `\` on Windows),
            // and is Debug-formatted, so `\` is escaped as `\\` on Windows.
            if msg.find("loadout/template.ron").is_none()
                && msg.find("loadout\\\\template.ron").is_none()
            {
                panic!("error message doesn't contain path:\n{msg}");
            }
        })
    }

    // -- Multi-pack tests (P0: N-source generalization)

    #[test]
    fn test_read_pack_priority() {
        // Pack2 (appended later) overrides Pack1; Pack1 overrides default.
        FileSystem::scope_with_packs(&|fs, main_path, pack1, pack2| {
            FileSystem::mock_file(main_path, "template.ron", "(default)");
            FileSystem::mock_file(pack1, "template.ron", "(pack1)");
            FileSystem::mock_file(pack2, "template.ron", "(pack2)");

            assert_eq!(fs.read_to_str("template", "ron"), "(pack2)");
        })
    }

    #[test]
    fn test_read_pack_falls_back_across_layers() {
        // Missing in pack2 but present in pack1 -> pack1 wins.
        // Missing in all packs -> default.
        FileSystem::scope_with_packs(&|fs, main_path, pack1, _pack2| {
            FileSystem::mock_file(main_path, "main.ron", "(default)");
            FileSystem::mock_file(pack1, "low.ron", "(pack1)");

            assert_eq!(fs.read_to_str("low", "ron"), "(pack1)");
            assert_eq!(fs.read_to_str("main", "ron"), "(default)");
            assert!(fs.read("nope", "ron").is_err());
        })
    }

    #[test]
    fn test_read_dir_multi_pack_merge() {
        // Files from all sources are merged; duplicates resolve to the
        // highest-priority pack.
        FileSystem::scope_with_packs(&|fs, main_path, pack1, pack2| {
            #[rustfmt::skip]
            FileSystem::mock_tree(main_path, vec![
                FsNode::Dir("entity", vec![
                    FsNode::File("shared.ron", "(default)"),
                    FsNode::File("main_only.ron", "(7)"),
                ]),
            ]);
            #[rustfmt::skip]
            FileSystem::mock_tree(pack1, vec![
                FsNode::Dir("entity", vec![
                    FsNode::File("shared.ron", "(pack1)"),
                    FsNode::File("pack1_only.ron", "(1)"),
                ]),
            ]);
            #[rustfmt::skip]
            FileSystem::mock_tree(pack2, vec![
                FsNode::Dir("entity", vec![
                    FsNode::File("shared.ron", "(pack2)"),
                    FsNode::File("pack2_only.ron", "(2)"),
                ]),
            ]);

            let mut files: Vec<String> = vec![];
            let _ = fs.read_dir("entity", &mut |e: DirEntry| match e {
                DirEntry::File(path, ext) => files.push(format!("{path}.{ext}")),
                DirEntry::Directory(path) => files.push(format!("{path}/")),
            });
            files.sort();
            assert_eq!(files, vec![
                "entity.main_only.ron".to_owned(),
                "entity.pack1_only.ron".to_owned(),
                "entity.pack2_only.ron".to_owned(),
                // shared resolves to the highest-priority pack (pack2)
                "entity.shared.ron".to_owned(),
            ]);
            assert_eq!(fs.read_to_str("entity.shared", "ron"), "(pack2)");
        })
    }

    #[test]
    fn test_exists_across_packs() {
        FileSystem::scope_with_packs(&|fs, main_path, pack1, _pack2| {
            FileSystem::mock_file(main_path, "main.ron", "(default)");
            FileSystem::mock_file(pack1, "pack.ron", "(pack1)");

            assert!(fs.exists(DirEntry::File("main", "ron")));
            assert!(fs.exists(DirEntry::File("pack", "ron")));
            assert!(!fs.exists(DirEntry::File("nope", "ron")));
        })
    }
}

#[cfg(test)]
mod integration {
    use super::{tests::*, *};
    use assets_manager::{Asset, AssetCache, FileAsset, SharedString};
    use hashbrown::HashSet;
    use serde::Deserialize;
    use std::borrow::Cow;

    #[derive(Deserialize, Clone, Debug, PartialEq)]
    struct WowManifest {
        prefix: usize,
    }

    #[derive(Deserialize, Clone, Debug, PartialEq)]
    struct WowFragment {
        pieces: Vec<usize>,
    }

    #[derive(Deserialize, Clone, Debug, PartialEq)]
    struct WowAsset {
        prefix: usize,
        pieces: HashSet<usize>,
    }

    impl FileAsset for WowManifest {
        const EXTENSION: &'static str = "ron";

        fn from_bytes(bytes: Cow<[u8]>) -> Result<Self, BoxedError> {
            assets_manager::asset::load_ron(&bytes)
        }
    }

    impl FileAsset for WowFragment {
        const EXTENSION: &'static str = "json";

        fn from_bytes(bytes: Cow<[u8]>) -> Result<Self, BoxedError> {
            assets_manager::asset::load_json(&bytes)
        }
    }

    // Pattern trying to simulate our i18n bundle
    impl crate::Asset for WowAsset {
        fn load(cache: &AssetCache, path: &SharedString) -> Result<Self, BoxedError> {
            let manifest = cache
                .load::<WowManifest>(&[path, ".", "_manifest"].concat())?
                .cloned();

            let mut total_pieces = HashSet::new();

            for id in cache.load_rec_dir::<WowFragment>(path)?.read().ids() {
                match cache.load::<WowFragment>(id) {
                    Ok(handle) => {
                        let WowFragment { pieces } = &handle.read().clone();
                        for piece in pieces {
                            if !total_pieces.insert(*piece) {
                                panic!("duplicate piece ({piece}) in: {id}");
                            }
                        }
                    },
                    // In i18n we warn here, but panics are more visible for
                    // tests, and errors shouldn't really be happening here.
                    //
                    // Probably.
                    Err(err) => panic!("error during loading: {id}\n{err:#?}"),
                }
            }

            Ok(Self {
                prefix: manifest.prefix,
                pieces: total_pieces,
            })
        }
    }

    #[test]
    fn test_read_dir() {
        FileSystem::scope(&|fs, main_path, _override_path| {
            #[rustfmt::skip]
            FileSystem::mock_tree(main_path, vec![
                FsNode::Dir("entity", vec![
                    FsNode::File("_manifest.ron", "(prefix: 5)"),
                    FsNode::File("first.json", r#"{"pieces": [1, 2]}"#),
                    FsNode::File("second.json", r#"{"pieces": [3, 4]}"#),
                ]),
            ]);

            let cache = AssetCache::with_source(fs);
            let asset = WowAsset::load(&cache, &"entity".into()).unwrap();
            assert_eq!(asset, WowAsset {
                prefix: 5,
                pieces: [1, 2, 3, 4].into(),
            });
        })
    }

    #[test]
    fn test_read_dir_override() {
        FileSystem::scope(&|fs, main_path, override_path| {
            #[rustfmt::skip]
            FileSystem::mock_tree(main_path, vec![
                FsNode::Dir("entity", vec![
                    FsNode::File("_manifest.ron", "(prefix: 5)"),
                    FsNode::File("first.json", r#"{"pieces": [1, 2]}"#),
                    FsNode::File("second.json", r#"{"pieces": [3, 4]}"#),
                ]),
            ]);

            #[rustfmt::skip]
            FileSystem::mock_tree(override_path, vec![
                FsNode::Dir("entity", vec![
                    FsNode::File("_manifest.ron", "(prefix: 5)"),
                    FsNode::File("first.json", r#"{"pieces": [5, 6]}"#),
                    FsNode::File("second.json", r#"{"pieces": [3, 4]}"#),
                ]),
            ]);

            let cache = AssetCache::with_source(fs);
            let asset = WowAsset::load(&cache, &"entity".into()).unwrap();
            assert_eq!(asset, WowAsset {
                prefix: 5,
                pieces: [5, 6, 3, 4].into(),
            });
        })
    }

    #[test]
    fn test_read_dir_partial_override() {
        FileSystem::scope(&|fs, main_path, override_path| {
            #[rustfmt::skip]
            FileSystem::mock_tree(main_path, vec![
                FsNode::Dir("entity", vec![
                    FsNode::File("_manifest.ron", "(prefix: 5)"),
                    FsNode::File("first.json", r#"{"pieces": [1, 2]}"#),
                    FsNode::File("second.json", r#"{"pieces": [3, 4]}"#),
                ]),
            ]);

            #[rustfmt::skip]
            FileSystem::mock_tree(override_path, vec![
                FsNode::Dir("entity", vec![
                    FsNode::File("_manifest.ron", "(prefix: 5)"),
                    // overriding only one of the files
                    FsNode::File("first.json", r#"{"pieces": [5, 6]}"#),
                ]),
            ]);

            let cache = AssetCache::with_source(fs);
            let asset = WowAsset::load(&cache, &"entity".into()).unwrap();
            assert_eq!(asset, WowAsset {
                prefix: 5,
                pieces: [5, 6, 3, 4].into(),
            });
        })
    }

    #[test]
    fn test_read_dir_partial_nested_override() {
        FileSystem::scope(&|fs, main_path, override_path| {
            #[rustfmt::skip]
            FileSystem::mock_tree(main_path, vec![
                FsNode::Dir("entity", vec![
                    FsNode::File("_manifest.ron", "(prefix: 5)"),
                    FsNode::Dir("nest", vec![
                        FsNode::File("first.json", r#"{"pieces": [1, 2]}"#),
                        FsNode::File("second.json", r#"{"pieces": [3, 4]}"#),
                    ]),
                ]),
            ]);

            #[rustfmt::skip]
            FileSystem::mock_tree(override_path, vec![
                FsNode::Dir("entity", vec![
                    FsNode::File("_manifest.ron", "(prefix: 7)"),
                    FsNode::Dir("nest", vec![
                        // overriding only one file, nested into directory
                        FsNode::File("first.json", r#"{"pieces": [5, 6]}"#),
                    ]),
                ]),
            ]);

            let cache = AssetCache::with_source(fs);
            let asset = WowAsset::load(&cache, &"entity".into()).unwrap();
            assert_eq!(asset, WowAsset {
                prefix: 7,
                pieces: [5, 6, 3, 4].into(),
            });
        })
    }
}
