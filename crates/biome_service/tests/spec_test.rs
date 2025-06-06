use biome_fs::{BiomePath, OsFileSystem};
use biome_service::workspace::{
    GetFileContentParams, OpenProjectParams, ScanKind, ScanProjectFolderParams,
    UpdateSettingsParams, server,
};
use camino::Utf8PathBuf;

/// Returns the path to the `fixtures/` directory, regardless of working dir.
fn get_fixtures_path() -> Utf8PathBuf {
    let mut path: Utf8PathBuf = std::env::current_dir().unwrap().try_into().unwrap();
    while !path.join("Cargo.lock").exists() {
        path = path
            .parent()
            .expect("couldn't find Cargo.lock")
            .to_path_buf();
    }
    path.join("crates/biome_service/tests/fixtures")
}

// If this test hangs, it probably recurses on one of the symlinks, such as
// `fixtures/shared/node_modules/common`.
#[test]
fn test_scanner_only_loads_type_definitions_from_node_modules() {
    let fixtures_path = get_fixtures_path();
    let fs = OsFileSystem::new(fixtures_path.clone());

    let workspace = server(Box::new(fs), None);
    let project_key = workspace
        .open_project(OpenProjectParams {
            path: fixtures_path.clone().into(),
            open_uninitialized: true,
        })
        .unwrap();

    workspace
        .update_settings(UpdateSettingsParams {
            project_key,
            configuration: Default::default(),
            workspace_directory: Some(fixtures_path.clone().into()),
        })
        .unwrap();

    workspace
        .scan_project_folder(ScanProjectFolderParams {
            project_key,
            path: None,
            watch: false,
            force: false,
            scan_kind: ScanKind::Project,
        })
        .unwrap();

    // FIXME: We should load manifests again in order to actually resolve
    //        type definitions, but last time we tried we ran into performance
    //        issues, and a reoccurrence of panics inside `node_semver` (#5829).
    //let manifest_result = workspace.get_file_content(GetFileContentParams {
    //    project_key,
    //    path: BiomePath::new(format!("{fixtures_path}/node_modules/shared/package.json")),
    //});
    //
    //assert!(
    //    manifest_result.is_ok_and(|result| !result.is_empty()),
    //    "package.json should be loaded"
    //);

    let d_mts_result = workspace.get_file_content(GetFileContentParams {
        project_key,
        path: BiomePath::new(format!(
            "{fixtures_path}/node_modules/shared/dist/index.d.mts"
        )),
    });

    assert!(
        d_mts_result.is_ok_and(|result| !result.is_empty()),
        "Type definitions should be loaded"
    );

    let js_result = workspace.get_file_content(GetFileContentParams {
        project_key,
        path: BiomePath::new(format!("{fixtures_path}/node_modules/shared/dist/index.js")),
    });

    assert!(js_result.is_err(), "JS file should be ignored");
}
