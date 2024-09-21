use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use color_print::cformat;

use crate::{
    cli::ExitCode,
    client::{Client, ClientError},
    config::save_config,
    image::load_and_save_image,
    kp,
};

pub(crate) async fn manga_download(
    client: &mut Client,
    console: &crate::term::Terminal,
    slug: impl Into<String>,
    volume: u32,
    parallel: bool,
) -> ExitCode {
    let slug: String = slug.into();
    console.info(cformat!("Downloading manga <m,s>{}</>...", &slug));

    match client.get_contents(&slug, volume.try_into().unwrap()).await {
        Err(e) => {
            console.error(cformat!("Failed to initiate download: <r,s>{}</>", e));
            1
        }
        Ok(contents) => {
            save_config(client.get_config());

            if contents.episodes.is_empty() {
                console.info("No episodes found!");
                return 0;
            }

            console.info(cformat!(
                "Found <m,s>{}</> episodes in volume <m,s>{}</>.",
                contents.episodes.len(),
                volume
            ));

            match kp::hash_to_aes_key(client.get_private_key(), &contents.hash) {
                Ok(aes_key) => {
                    // Download all the images
                    console.log(format!("AES Key generated successfully: {:?}", &aes_key));
                    console.log(format!("Original AES hash: {}", &contents.hash));
                    for episode in contents.episodes.iter() {
                        let ep_dir =
                            get_output_directory(&slug, volume, Some(episode.episode), true);

                        console.info(cformat!(
                            "  Downloading episode <m,s>{}</> for <m,s>{}</>...",
                            episode.episode,
                            &slug,
                        ));

                        let progress = Arc::new(
                            console.make_progress(episode.pages.len() as u64, Some("Downloading")),
                        );

                        if parallel {
                            let tasks: Vec<_> = episode
                                .pages
                                .iter()
                                .enumerate()
                                .map(|(idx, image)| {
                                    let wrap_client = client.clone();
                                    let image_dir = ep_dir.clone();
                                    let cnsl = console.clone();
                                    let image = image.clone();
                                    let key = aes_key.clone();
                                    let progress = Arc::clone(&progress);

                                    tokio::spawn(async move {
                                        match actual_downloader(
                                            DownloadNode {
                                                client: wrap_client,
                                                url: image.url.clone(),
                                                idx,
                                                key,
                                            },
                                            cnsl.clone(),
                                            &image_dir,
                                            progress,
                                        )
                                        .await
                                        {
                                            Ok(_) => {}
                                            Err(e) => {
                                                cnsl.error(format!(
                                                    "    Failed to download <m,s>{}</>: <r,s>{}</>",
                                                    image.url, e
                                                ));
                                            }
                                        }
                                    })
                                })
                                .collect();

                            futures::future::join_all(tasks).await;
                        } else {
                            for (idx, image) in episode.pages.iter().enumerate() {
                                let node = DownloadNode {
                                    client: client.clone(),
                                    url: image.url.clone(),
                                    idx,
                                    key: aes_key.clone(),
                                };

                                match actual_downloader(
                                    node,
                                    console.clone(),
                                    &ep_dir,
                                    progress.clone(),
                                )
                                .await
                                {
                                    Ok(_) => {}
                                    Err(e) => {
                                        console.error(format!(
                                            "    Failed to download <m,s>{}</>: <r,s>{}</>",
                                            image.url, e
                                        ));
                                    }
                                }
                            }
                        }

                        progress.finish();
                    }

                    0
                }
                Err(e) => {
                    console.error(cformat!(
                        "Failed to generate make key: <r,s>{}</>\n<s>Hash</s>: {}",
                        e,
                        &contents.hash
                    ));
                    1
                }
            }
        }
    }
}

fn get_output_directory(
    slug: &str,
    volume: u32,
    chapter: Option<i32>,
    create_folder: bool,
) -> PathBuf {
    let cwd = std::env::current_dir().unwrap();
    let mut pathing = cwd.join("DOWNLOADS");
    pathing.push(slug);

    pathing.push(format!("v{:02}", volume));

    if let Some(chapter) = chapter {
        pathing.push(format!("c{:03}", chapter));
    }

    if create_folder {
        std::fs::create_dir_all(&pathing).unwrap();
    }

    pathing
}

struct DownloadNode {
    client: Client,
    url: String,
    idx: usize,
    key: Vec<u8>,
}

async fn actual_downloader(
    node: DownloadNode,
    console: crate::term::Terminal,
    path: &Path,
    progress: Arc<indicatif::ProgressBar>,
) -> Result<(), ClientError> {
    if console.is_debug() {
        console.log(cformat!(
            "   Downloading <m,s>{}</> to <m,s>{}</>...",
            &node.url,
            path.display()
        ));
    }

    let image_fn = format!("p{:03}.bin", node.idx);
    let img_dl_path = path.join(&image_fn);

    match node.client.download_image(&node.url).await {
        Ok(dyn_image) => {
            // Decrypt the image and save
            match load_and_save_image(&dyn_image, &node.key, &img_dl_path) {
                Ok(_) => {
                    progress.inc(1);
                    Ok(())
                }
                Err(err) => {
                    console.error(format!(
                        "    Failed to save <m,s>{}</>: <r,s>{}</>",
                        &node.url, err
                    ));
                    Err(ClientError::Image(err))
                }
            }
        }
        Err(err) => {
            console.error(format!(
                "    Failed to download <m,s>{}</>: <r,s>{}</>",
                &node.url, err
            ));

            Err(err)
        }
    }
}
