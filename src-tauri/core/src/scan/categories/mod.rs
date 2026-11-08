//! The whitelist: things that rebuild themselves or that the user explicitly opts into.
//! One file per category; the order here is the order in the plan.

mod app_caches;
mod browsers;
mod dev_tools;
mod dism;
mod dns;
mod docker_all;
mod docker_cache;
mod driver_installers;
mod driver_store;
mod error_reports;
mod internet_cache;
mod jetbrains;
mod recycle_bin;
mod shaders;
mod system_logs;
mod temp;
mod thumbnails;
mod windows_old;
mod windows_update;

use std::path::{Path, PathBuf};

use super::category::Category;
use crate::ctx::Ctx;
use crate::system::command;

/// All categories available on this computer.
pub fn all(ctx: &Ctx) -> Vec<Box<dyn Category>> {
    let u = ctx.user;
    let mut list: Vec<Box<dyn Category>> = vec![
        Box::new(temp::new(u)),
        Box::new(internet_cache::new(u)),
        Box::new(recycle_bin::RecycleBin::default()),
        Box::new(error_reports::new(u)),
        Box::new(windows_update::new(u)),
        Box::new(system_logs::new(u)),
        Box::new(browsers::new(u)),
        Box::new(dev_tools::new(u)),
        Box::new(app_caches::new(u)),
        Box::new(jetbrains::new(u)),
        Box::new(driver_installers::new(u)),
        Box::new(shaders::new(u)),
        Box::new(thumbnails::Thumbnails),
        Box::new(dns::Dns),
        Box::new(windows_old::WindowsOld),
        Box::new(dism::Dism),
    ];
    if cfg!(windows) {
        list.push(Box::new(driver_store::DriverStore::default()));
    }
    if command::on_path("docker") {
        list.push(Box::new(docker_all::DockerAll));
        list.push(Box::new(docker_cache::DockerCache));
    }
    list
}

/// Sub-folders of `dir` (no links), sorted - for profiles and versioned app folders.
pub(crate) fn subfolders(dir: &Path) -> Vec<PathBuf> {
    let Ok(rd) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut v: Vec<PathBuf> = rd
        .flatten()
        .filter(|e| e.metadata().is_ok_and(|m| m.is_dir() && !crate::fs::link::is_link(&m)))
        .map(|e| e.path())
        .collect();
    v.sort();
    v
}
