use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use crate::core::clients::client::ClientType;
use crate::{log_debug, log_warn};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedClient {
    pub main_class: String,
    pub client_type: ClientType,
    pub confidence: u8,
    pub reason: String,
}

#[allow(dead_code)]
const FORGE_MAIN_CANDIDATES: &[&str] = &[
    "net.minecraftforge.legacy.server.start.LegacyServerStart",
    "GradleStart",
    "net.minecraftforge.fml.common.launcher.FMLServerTweaker",
    "cpw.mods.bootstrap.Bootstrapper",
    "cpw.mods.fml.common.launcher.FMLTweaker",
    "net.minecraftforge.fml.common.launcher.FMLTweaker",
];

const FORGE_LAUNCHER_HINTS: &[&str] = &[
    "net/minecraft/launchwrapper/Launch",
    "net/minecraft/launchwrapper/LaunchClassLoader",
    "cpw/mods/fml/common/launcher/FMLTweaker",
    "net/minecraftforge/fml/common/launcher/FMLTweaker",
    "net/minecraftforge/legacy/ForgeTweaker",
];

const FABRIC_LAUNCHER_HINTS: &[&str] = &[
    "net/fabricmc/loader/impl/launch/knot/KnotClient",
    "net/fabricmc/loader/impl/launch/knot/Knot",
    "fabric/loader/impl/launch/knot/KnotClient",
    "net/fabricmc/loader/impl/launch/knot/KnotServer",
];

const VANILLA_MAIN_CANDIDATES: &[&str] = &["net/minecraft/client/main/Main"];

const FORGE_DEPENDENCY_HINTS: &[&str] = &[
    "forge_universal",
    "forgeSrc",
    "net/minecraftforge",
    "cpw/mods/fml",
];

const FABRIC_DEPENDENCY_HINTS: &[&str] = &["fabric-api", "fabric-loader", "net/fabricmc"];

const OPTIFINE_HINT: &str = "optifine/OptiFineClassTransformer";

const LWJGL_HINTS: &[&str] = &["org/lwjgl", "Lwjgl"];

pub fn detect_client_type(jar_path: &Path) -> Option<DetectedClient> {
    let file = match File::open(jar_path) {
        Ok(f) => f,
        Err(e) => {
            log_warn!("Cannot open {}: {}", jar_path.display(), e);
            return None;
        }
    };
    let reader = BufReader::new(file);
    let mut archive = match zip::ZipArchive::new(reader) {
        Ok(a) => a,
        Err(e) => {
            log_warn!("{} is not a valid zip/jar: {}", jar_path.display(), e);
            return None;
        }
    };

    let manifest_main_class = read_manifest_main_class(&mut archive);

    let mut class_names: Vec<String> = Vec::new();
    let mut has_forge_class = false;
    let mut has_fabric_class = false;
    let mut has_optifine = false;
    let mut has_launchwrapper = false;
    let mut has_lwjgl = false;
    let mut has_vanilla_main = false;

    for i in 0..archive.len() {
        let entry = match archive.by_index(i) {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry.is_file() {
            continue;
        }
        let name = match entry.name().to_string() {
            n if n.len() < 8 => continue,
            n => n,
        };
        if !name.ends_with(".class") {
            continue;
        }
        if name.len() > 4096 {
            continue;
        }

        for hint in FORGE_LAUNCHER_HINTS {
            if name.contains(hint) {
                has_forge_class = true;
                has_launchwrapper = true;
                break;
            }
        }
        for hint in FABRIC_LAUNCHER_HINTS {
            if name.contains(hint) {
                has_fabric_class = true;
                break;
            }
        }
        if name.contains(OPTIFINE_HINT) {
            has_optifine = true;
        }
        for hint in VANILLA_MAIN_CANDIDATES {
            if name.contains(hint) {
                has_vanilla_main = true;
            }
        }
        for hint in LWJGL_HINTS {
            if name.contains(hint) {
                has_lwjgl = true;
            }
        }

        if class_names.len() < 4000 {
            class_names.push(name);
        }

        if has_launchwrapper && has_fabric_class {
            break;
        }
    }

    let mut deps: HashMap<String, String> = HashMap::new();
    if let Ok(data) = read_file_from_archive(&mut archive, "dependencies.json") {
        if let Ok(map) = serde_json::from_slice::<HashMap<String, String>>(&data) {
            deps = map;
        }
    }

    let has_forge_dep = deps
        .values()
        .any(|v| FORGE_DEPENDENCY_HINTS.iter().any(|h| v.contains(h)));
    let has_fabric_dep = deps
        .values()
        .any(|v| FABRIC_DEPENDENCY_HINTS.iter().any(|h| v.contains(h)));

    log_debug!(
        "Detection: forge={} fabric={} optifine={} launchwrapper={} vanilla={} lwjgl={} forge_dep={} fabric_dep={}",
        has_forge_class,
        has_fabric_class,
        has_optifine,
        has_launchwrapper,
        has_vanilla_main,
        has_lwjgl,
        has_forge_dep,
        has_fabric_dep,
    );

    if let Some(main) = &manifest_main_class {
        log_debug!("Manifest Main-Class: {}", main);
    }

    if has_fabric_class || has_fabric_dep {
        let main = manifest_main_class
            .clone()
            .unwrap_or_else(|| "net.fabricmc.loader.impl.launch.knot.KnotClient".to_string());
        return Some(DetectedClient {
            main_class: main,
            client_type: ClientType::Fabric,
            confidence: if has_fabric_class && has_fabric_dep { 100 } else { 80 },
            reason: "Detected fabric loader classes/dependencies".to_string(),
        });
    }

    if has_forge_class || has_launchwrapper || has_forge_dep {
        let main = if let Some(m) = &manifest_main_class {
            if m.contains("launchwrapper")
                || m == "GradleStart"
                || m == "net.minecraft.launchwrapper.Launch"
            {
                m.clone()
            } else if has_optifine {
                "net.minecraft.launchwrapper.Launch".to_string()
            } else {
                m.clone()
            }
        } else {
            "net.minecraft.launchwrapper.Launch".to_string()
        };

        if !main.contains("launchwrapper") && !main.contains("GradleStart") && !has_optifine {
            log_warn!(
                "Forge-like client detected but main class is '{}' which is not a known launcher; falling back to vanilla main",
                main
            );
        } else {
            return Some(DetectedClient {
                main_class: "net.minecraft.launchwrapper.Launch".to_string(),
                client_type: ClientType::Forge,
                confidence: if has_forge_class && has_forge_dep { 100 } else { 80 },
                reason: "Detected Forge launchwrapper / OptiFine-Forge / Forge deps".to_string(),
            });
        }
    }

    if has_optifine && !has_launchwrapper {
        let has_optifine_forge = class_names
            .iter()
            .any(|n| n.contains("optifine/reflect/ReflectorForge"));
        if has_optifine_forge {
            return Some(DetectedClient {
                main_class: "net.minecraft.launchwrapper.Launch".to_string(),
                client_type: ClientType::Forge,
                confidence: 85,
                reason: "OptiFine Forge integration detected (ReflectorForge class)"
                    .to_string(),
            });
        }
        let main = manifest_main_class.clone().unwrap_or_else(|| {
            "net.minecraft.client.main.Main".to_string()
        });
        return Some(DetectedClient {
            main_class: main,
            client_type: ClientType::Default,
            confidence: 60,
            reason: "OptiFine without Forge launchwrapper; using manifest or vanilla main".to_string(),
        });
    }

    if has_vanilla_main {
        let main = "net.minecraft.client.main.Main".to_string();
        return Some(DetectedClient {
            main_class: main,
            client_type: ClientType::Default,
            confidence: 90,
            reason: "Detected vanilla Main class".to_string(),
        });
    }

    if let Some(main) = manifest_main_class {
        return Some(DetectedClient {
            main_class: main,
            client_type: ClientType::Default,
            confidence: 30,
            reason: "Using manifest Main-Class (unknown structure)".to_string(),
        });
    }

    log_warn!("Could not detect client type for {}", jar_path.display());
    None
}

fn read_manifest_main_class(
    archive: &mut zip::ZipArchive<BufReader<File>>,
) -> Option<String> {
    let mut entry = archive
        .by_name("META-INF/MANIFEST.MF")
        .ok()?;
    let mut contents = String::new();
    entry.read_to_string(&mut contents).ok()?;
    for line in contents.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Main-Class:") {
            return Some(rest.trim().to_string());
        }
        if let Some(rest) = trimmed.strip_prefix("Main-Class ") {
            return Some(rest.trim().to_string());
        }
    }
    None
}

fn read_file_from_archive(
    archive: &mut zip::ZipArchive<BufReader<File>>,
    name: &str,
) -> Result<Vec<u8>, String> {
    let mut entry = archive
        .by_name(name)
        .map_err(|e| format!("entry not found: {e}"))?;
    let mut buf = Vec::new();
    entry
        .read_to_end(&mut buf)
        .map_err(|e| format!("read error: {e}"))?;
    Ok(buf)
}

#[allow(dead_code)]
pub fn describe(detected: &DetectedClient) -> String {
    format!(
        "{:?} main='{}' (confidence {}%, reason: {})",
        detected.client_type, detected.main_class, detected.confidence, detected.reason
    )
}
