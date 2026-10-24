use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImmichAlbum {
    pub id: String,
    pub album_name: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RandomSearch<'a> {
    pub album_ids: [&'a str; 1],
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub size: u32,
    pub with_exif: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImmichAsset {
    pub id: String,
    #[serde(default)]
    pub original_mime_type: Option<String>,
    #[serde(default)]
    pub exif_info: Option<ExifInfo>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExifInfo {
    #[serde(default)]
    pub exif_image_width: Option<f64>,
    #[serde(default)]
    pub exif_image_height: Option<f64>,
    #[serde(default)]
    pub orientation: Option<String>,
}

impl ImmichAsset {
    pub fn displayed_dims(&self) -> Option<(u32, u32)> {
        let exif = self.exif_info.as_ref()?;
        let width = exif.exif_image_width? as u32;
        let height = exif.exif_image_height? as u32;

        let rotated = exif
            .orientation
            .as_deref()
            .and_then(|orientation| orientation.trim().parse::<u8>().ok())
            .is_some_and(|orientation| (5..=8).contains(&orientation));

        let dims = if rotated {
            (height, width)
        } else {
            (width, height)
        };

        Some(dims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    const FIXTURE: &str = r#"[
      {
        "id": "a1",
        "type": "IMAGE",
        "originalMimeType": "image/heic",
        "exifInfo": { "exifImageWidth": 4032, "exifImageHeight": 3024, "orientation": "6" }
      },
      {
        "id": "a2",
        "type": "IMAGE",
        "originalMimeType": "image/jpeg",
        "exifInfo": { "exifImageWidth": 4032, "exifImageHeight": 3024, "orientation": "1" }
      },
      {
        "id": "a3",
        "type": "IMAGE",
        "originalMimeType": "image/png",
        "exifInfo": { "exifImageWidth": null, "exifImageHeight": null, "orientation": null }
      },
      { "id": "a4", "type": "IMAGE" }
    ]"#;

    fn assets() -> Vec<ImmichAsset> {
        serde_json::from_str(FIXTURE).unwrap()
    }

    #[test]
    fn a_random_search_response_parses() {
        let assets = assets();

        assert_eq!(assets.len(), 4);
        assert_eq!(assets[0].id, "a1");
        assert_eq!(assets[0].original_mime_type.as_deref(), Some("image/heic"));
        assert_eq!(assets[3].original_mime_type, None);
    }

    #[test]
    fn a_rotated_exif_orientation_swaps_the_dims() {
        let assets = assets();

        assert_eq!(assets[0].displayed_dims(), Some((3024, 4032)));
        assert_eq!(assets[1].displayed_dims(), Some((4032, 3024)));
    }

    #[test]
    fn missing_exif_dims_are_unknown() {
        let assets = assets();

        assert_eq!(assets[2].displayed_dims(), None);
        assert_eq!(assets[3].displayed_dims(), None);
    }

    #[test]
    fn a_random_search_serializes_in_camel_case() {
        let search = RandomSearch {
            album_ids: ["album-1"],
            kind: "IMAGE",
            size: 10,
            with_exif: true,
        };

        assert_eq!(
            serde_json::to_value(&search).unwrap(),
            serde_json::json!({ "albumIds": ["album-1"], "type": "IMAGE", "size": 10, "withExif": true })
        );
    }
}
