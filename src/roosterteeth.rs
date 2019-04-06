// Welcome to struct hell.
// It makes sense if you look at it in the form of a JSON file.

#[derive(Serialize, Deserialize)]
pub struct Pagination {
    pub page: u16,
    pub per_page: u16,
    pub total_pages: u16,
    pub total_results: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Videos {
    pub data: Vec<VideoInstance>,

    #[serde(flatten)]
    pub pagination: Pagination,
}

#[derive(Serialize, Deserialize)]
pub struct VideoInstance {
    pub id: u32,
    pub attributes: VideoAttributes,
    pub links: VideoLinks,
    pub canonical_links: VideoCanonical,
    pub included: VideosIncluded,
}

#[derive(Serialize, Deserialize)]
pub struct VideoAttributes {
    pub title: String,
    pub display_title: String,
    pub show_title: String,

    pub caption: String,
    pub description: String,

    pub slug: String,
    pub channel_slug: String,
    pub show_slug: String,
    pub season_slug: String,

    #[serde(rename = "number")]
    pub episode_number: u16,
    pub season_number: u16,

    pub length: u32,

    #[serde(rename = "member_golive_at")]
    pub release_public: String,
    #[serde(rename = "sponsor_golive_at")]
    pub release_sponsor: String,

    pub is_sponsors_only: bool,
    pub sort_number: u32,
}

#[derive(Serialize, Deserialize)]
pub struct VideoLinks {
    #[serde(rename = "self")]
    pub own: String,
    pub show: String,
    pub related_shows: String,
    pub channel: String,
    pub season: String,
    pub videos: String,
}

#[derive(Serialize, Deserialize)]
pub struct VideoCanonical {
    #[serde(rename = "self")]
    pub own: String,
    pub show: String,
}

#[derive(Serialize, Deserialize)]
pub struct VideosIncluded {
    pub images: Vec<VideoImage>,
}

#[derive(Serialize, Deserialize)]
pub struct VideoImage {
    pub id: u32,
    #[serde(rename = "type")]
    pub kind: String,
    pub attributes: VideoImageAttributes,
}

#[derive(Serialize, Deserialize)]
pub struct VideoImageAttributes {
    pub thumb: String,
    pub small: String,
    pub medium: String,
    pub large: String,
}

#[derive(Serialize, Deserialize)]
pub struct Shows {
    pub data: Vec<ShowInstance>,
}

#[derive(Serialize, Deserialize)]
pub struct ShowInstance {
    pub id: u16,
    pub attributes: ShowAttributes,
    pub links: ShowLinks,
    pub canonical_links: ShowCanonical,
    pub included: ShowIncluded,
}

#[derive(Serialize, Deserialize)]
pub struct ShowAttributes {
    pub title: String,
    pub slug: String,
    pub is_sponsors_only: bool,
    pub updated_at: String,
    pub published_at: String,
    pub summary: String,
    pub channel_slug: String,
    pub season_count: u16,
    pub episode_count: u16,
    #[serde(rename = "last_episode_golive_at")]
    pub last_update: String,
}

#[derive(Serialize, Deserialize)]
pub struct ShowLinks {
    #[serde(rename = "self")]
    pub own: String,
    pub seasons: String,
}

#[derive(Serialize, Deserialize)]
pub struct ShowCanonical {
    #[serde(rename = "self")]
    pub own: String,
}

#[derive(Serialize, Deserialize)]
pub struct ShowIncluded {
    pub images: Vec<ShowImage>,
}

#[derive(Serialize, Deserialize)]
pub struct ShowImage {
    pub id: u32,
    pub attributes: ShowImageAttributes,
}

#[derive(Serialize, Deserialize)]
pub struct ShowImageAttributes {
    pub thumb: String,
    pub small: String,
    pub medium: String,
    pub large: String,
    pub orientation: String,
    pub image_type: String,
}
