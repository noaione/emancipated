use color_print::cformat;

use crate::{
    cli::ExitCode,
    client::{Client, BASE_HOST},
    config::save_config,
    models::ComicTag,
    term::macros::linkify,
};

pub(crate) async fn manga_search(
    client: &mut Client,
    console: &crate::term::Terminal,
    query: impl Into<String>,
) -> ExitCode {
    let query: String = query.into();

    console.info(cformat!(
        "Searching for manga with query <m,s>{}</>...",
        &query
    ));

    match client.search(&query).await {
        Ok(results) => {
            save_config(client.get_config());

            if results.is_empty() {
                console.info("No results found!");
                return 0;
            }

            console.info(cformat!("Found <m,s>{}</> results:", results.len()));

            for (i, result) in results.iter().enumerate() {
                let manga_url = format!("https://{}/{}", &*BASE_HOST, result.slug);
                let linked = linkify!(manga_url, &result.title);

                let text_data = cformat!("<s>{}</> ({})", linked, result.slug);
                console.info(format!("  [{:02}] {}", i + 1, text_data,));
            }

            0
        }
        Err(e) => {
            console.error(cformat!("Failed to search: <r,s>{}</>", e));
            1
        }
    }
}

fn format_vec_comma(data: Vec<String>) -> String {
    // Format into a comma separated list, with last one being "and" instead of a comma
    if data.len() == 1 {
        return data[0].clone();
    }

    if data.len() == 2 {
        return format!("{} and {}", data[0], data[1]);
    }

    let before_last = data[..data.len() - 1].join(", ");
    let last = data[data.len() - 1].clone();

    format!("{}, and {}", before_last, last)
}

fn format_tags(tags: Vec<ComicTag>) -> String {
    let parsed_tags = tags
        .iter()
        .map(|tag| cformat!("<p(244),reverse,bold>{}</>", tag.name))
        .collect::<Vec<String>>();
    format_vec_comma(parsed_tags)
}

pub(crate) async fn manga_info(
    client: &mut Client,
    console: &crate::term::Terminal,
    slug: impl Into<String>,
) -> ExitCode {
    let slug: String = slug.into();
    console.info(cformat!(
        "Fetching info for manga <magenta,bold>{}</>...",
        slug
    ));

    match client.get_volumes(&slug).await {
        Ok(comic) => {
            save_config(client.get_config());

            let manga_url = format!("https://{}/{}", &*BASE_HOST, comic.comic.slug);
            let linked = linkify!(manga_url, &comic.comic.title);

            console.info(cformat!("Title information for <m,s>{}</>", linked));

            let joined_authors = format_vec_comma(comic.comic.metadata.creators);
            console.info(&cformat!("  <s>Authors</>: {}", joined_authors));

            console.info(&cformat!(
                "  <s>Tags</>: {}",
                format_tags(comic.comic.genres)
            ));

            if let Some(true) = comic.comic.metadata.completed {
                console.info(cformat!("  <s>Status</>: <g,s>Completed</>"));
            } else {
                console.info(cformat!("  <s>Status</>: <y,s>Ongoing</>"));
            }

            println!();

            console.info(&cformat!(
                "  <s>Volumes</>: {} volumes",
                comic.volumes.len()
            ));

            if !comic.volumes.is_empty() {
                for volume in comic.volumes.iter() {
                    let mut base_info = cformat!(
                        "    <s>{}</> (#{} - {})",
                        volume.name,
                        volume.number,
                        volume.slug
                    );

                    if volume.purchased {
                        base_info = cformat!("{} <g!,s>[<rev>Purchased</rev>]</g!,s>", base_info);
                    } else if let Some(price) = &volume.price {
                        base_info =
                            cformat!("{} <y!,s>[<rev>Price</rev>: {}]</y!,s>", base_info, price);
                    }

                    console.info(&base_info);
                    if let Some(publish_at) = &volume.release_at {
                        console.info(&cformat!("     <s>Release Date</>: {}", publish_at));
                    }
                }
            }

            0
        }
        Err(e) => {
            console.error(cformat!("Failed to fetch manga info: <r,s>{}</>", e));
            1
        }
    }
}
