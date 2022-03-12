use std::env;
use std::fmt::Write;
use std::path::PathBuf;
use log::info;
use tokio::io::AsyncWriteExt;
use crate::core::FileInfo;

/// setting log config
pub async fn setting_log(output: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file_info = FileInfo::from_vec(vec![output, "blueberry_box.log"]).await;
    if !file_info.directory_exist() {
        file_info.create_directory().await;
    }
    let log_output = file_info.path_buf.as_path();
    fern::Dispatch::new()
        // Perform allocation-free log formatting
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}",
                message
            ))
            // out.finish(format_args!(
            //     "{}[{}][{}] {}",
            //     chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
            //     record.target(),
            //     record.level(),
            //     message
            // ))
        })
        // Add blanket level filter -
        .level(log::LevelFilter::Debug)
        // - and per-module overrides
        .level_for("hyper", log::LevelFilter::Info)
        // Output to stdout, files, and other Dispatch configurations
        .chain(std::io::stdout())
        .chain(fern::log_file(log_output)?)
        // Apply globally
        .apply()?;
    Ok(())
}

pub async fn output_config_json() -> Result<(), Box<dyn std::error::Error>> {
    let json = r##"[
    {
        "path":"",
        "metas":[
            {
                "name":"",
                "content":""
            }
        ],
        "title":"",
        "heads":[
        ]
    }
]"##;

    let json_doc = r##"path supports three rules: pre:// tail:// regular://; example pre://index/home
metas Head SEO Resource Node
title Web Page title
heads Custom head"##;
    let mut output_json = PathBuf::from(env::current_dir()?);
    output_json.push("config.json");
    let mut output_json_doc = PathBuf::from(env::current_dir()?);
    output_json_doc.push("config_doc.txt");

    if !std::path::Path::new(&output_json).exists() {
        let mut file = tokio::fs::File::create(output_json).await?;
        file.write_all(json.as_bytes()).await?;
        info!("config.json out success!")
    }

    if !std::path::Path::new(&output_json_doc).exists() {
        let mut file = tokio::fs::File::create(output_json_doc).await?;
        file.write_all(json_doc.as_bytes()).await?;
        info!("config_doc.txt out success!")
    }

    Ok(())
}
