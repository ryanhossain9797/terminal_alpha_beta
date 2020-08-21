use super::*;
use mongodb::bson::{doc, Bson};

pub async fn get(title: String, pass: String) -> anyhow::Result<Option<String>> {
    Ok(
        match database::mongo::get_mongo()
            .await
            .ok_or_else(|| anyhow::anyhow!("Couldn't fetch db connection"))?
            //If db connection is successful
            .collection("info")
            //Search for required info with title and pass
            .find_one(
                doc! {
                    "title": &title,
                    "pass": &pass,
                },
                None,
            )
            .await?
        {
            //If a valid document is found
            Some(doc) =>
            //Get info data
            {
                doc.get("info")
                    .and_then(Bson::as_str)
                    .ok_or_else(|| anyhow::anyhow!("Couldn't fetch info"))?
                    .to_string()
                    .into()
            }
            //If no valid document is found
            _ => None,
        },
    )
}
