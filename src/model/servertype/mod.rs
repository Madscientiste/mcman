use crate::app::{App, Resolvable, ResolvedFile};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::model::Downloadable;

pub mod interactive;
pub mod meta;
pub mod parse;

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub enum SoftwareType {
    Normal,
    Modded,
    Proxy,
    #[default]
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ServerType {
    Vanilla {},

    PaperMC {
        project: String,
        #[serde(default = "latest")]
        #[serde(skip_serializing_if = "crate::util::is_default_str")]
        build: String,
    },

    Purpur {
        #[serde(default = "latest")]
        build: String,
    },

    Fabric {
        #[serde(default = "latest")]
        loader: String,

        #[serde(default = "latest")]
        installer: String,
    },

    Quilt {
        #[serde(default = "latest")]
        loader: String,

        #[serde(default = "latest")]
        installer: String,
    },

    NeoForge {
        #[serde(default = "latest")]
        loader: String,
    },

    Forge {
        #[serde(default = "latest")]
        loader: String,
    },

    BuildTools {
        #[serde(default = "spigot")]
        software: String,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        #[serde(default = "Vec::new")]
        args: Vec<String>,
    },

    Paper {},
    Velocity {},
    Waterfall {},
    BungeeCord {},

    Downloadable {
        #[serde(flatten)]
        inner: Downloadable,
    },
}

#[derive(Debug)]
pub enum InstallMethod {
    SingleJar,
    Installer {
        name: String,
        label: String,
        args: Vec<String>,
        rename_from: Option<String>,
        jar_name: String,
    },
}

impl ServerType {
    pub fn get_software_type(&self) -> SoftwareType {
        match self {
            Self::Velocity {} | Self::BungeeCord {} | Self::Waterfall {} => SoftwareType::Proxy,
            Self::PaperMC { project, .. } if project == "velocity" || project == "waterfall" => {
                SoftwareType::Proxy
            }
            Self::Quilt { .. }
            | Self::Fabric { .. }
            | Self::NeoForge { .. }
            | Self::Forge { .. } => SoftwareType::Modded,
            Self::Vanilla {}
            | Self::Paper {}
            | Self::PaperMC { .. }
            | Self::Purpur { .. }
            | Self::BuildTools { .. } => SoftwareType::Normal,
            Self::Downloadable { .. } => SoftwareType::Unknown,
        }
    }

    pub fn get_modrinth_name(&self) -> Option<String> {
        match self {
            ServerType::Fabric { .. } => Some("fabric"),
            ServerType::Quilt { .. } => Some("quilt"),
            ServerType::Forge { .. } => Some("forge"),
            ServerType::NeoForge { .. } => Some("neoforge"),
            ServerType::Paper {} => Some("paper"),
            ServerType::BuildTools { .. } => Some("spigot"),
            ServerType::Purpur { .. } => Some("purpur"),
            ServerType::BungeeCord {} => Some("bungeecord"),
            ServerType::Velocity {} => Some("velocity"),
            ServerType::Waterfall {} => Some("waterfall"),
            ServerType::PaperMC { project, .. } => Some(project.as_str()),
            _ => None,
        }
        .map(ToOwned::to_owned)
    }

    pub fn is_modded(&self) -> bool {
        self.get_software_type() == SoftwareType::Modded
    }

    pub fn supports_eula_args(&self) -> bool {
        !matches!(self, Self::Vanilla {}) && !self.is_modded()
    }
}

impl ToString for ServerType {
    fn to_string(&self) -> String {
        match self {
            ServerType::Vanilla {} => String::from("Vanilla"),
            ServerType::PaperMC { project, build } => format!("{project} build {build}"),
            ServerType::Purpur { build } => format!("Purpur build {build}"),
            ServerType::Fabric { loader, .. } => format!("Fabric {loader}"),
            ServerType::Quilt { loader, .. } => format!("Quilt {loader}"),
            ServerType::NeoForge { loader } => format!("NeoForge {loader}"),
            ServerType::Forge { loader } => format!("Forge {loader}"),
            ServerType::BuildTools { software, .. } => format!("(BuildTools) {software}"),
            ServerType::Paper {} => "Paper".to_owned(),
            ServerType::Velocity {} => "Velocity".to_owned(),
            ServerType::Waterfall {} => "Waterfall".to_owned(),
            ServerType::BungeeCord {} => "BungeeCord".to_owned(),
            ServerType::Downloadable { inner } => inner.to_string(),
        }
    }
}

#[async_trait]
impl Resolvable for ServerType {
    async fn resolve_source(&self, app: &App) -> Result<ResolvedFile> {
        let version = &app.mc_version();

        match self {
            ServerType::Vanilla {} => app.vanilla().resolve_source(version).await,
            ServerType::PaperMC { project, build } => {
                app.papermc().resolve_source(project, version, build).await
            }
            ServerType::Purpur { build } => app.purpur().resolve_source(version, build).await,
            ServerType::Fabric { loader, installer } => {
                app.fabric().resolve_source(loader, installer).await
            }
            ServerType::Quilt { installer, .. } => app.quilt().resolve_installer(installer).await,
            ServerType::NeoForge { loader } => app.neoforge().resolve_source(loader).await,
            ServerType::Forge { loader } => app.forge().resolve_source(loader).await,
            ServerType::BuildTools { .. } => buildtools().resolve_source(app).await,
            ServerType::Paper {} => {
                app.papermc()
                    .resolve_source("paper", version, "latest")
                    .await
            }
            ServerType::Velocity {} => {
                app.papermc()
                    .resolve_source("velocity", version, "latest")
                    .await
            }
            ServerType::Waterfall {} => {
                app.papermc()
                    .resolve_source("waterfall", version, "latest")
                    .await
            }
            ServerType::BungeeCord {} => bungeecord().resolve_source(app).await,
            ServerType::Downloadable { inner } => inner.resolve_source(app).await,
        }
    }
}

fn latest() -> String {
    "latest".to_owned()
}

fn spigot() -> String {
    "spigot".to_owned()
}

static BUNGEECORD_JENKINS: &str = "https://ci.md-5.net";
static BUNGEECORD_JOB: &str = "BungeeCord";
static BUNGEECORD_ARTIFACT: &str = "BungeeCord";
static BUILDTOOLS_JENKINS: &str = "https://hub.spigotmc.org/jenkins";

pub fn bungeecord() -> Downloadable {
    Downloadable::Jenkins {
        url: BUNGEECORD_JENKINS.to_owned(),
        job: BUNGEECORD_JOB.to_owned(),
        build: "latest".to_owned(),
        artifact: BUNGEECORD_ARTIFACT.to_owned(),
    }
}

pub fn buildtools() -> Downloadable {
    Downloadable::Jenkins {
        url: BUILDTOOLS_JENKINS.to_owned(),
        job: "BuildTools".to_owned(),
        build: "latest".to_owned(),
        artifact: "first".to_owned(),
    }
}
