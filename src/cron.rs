use chrono::Local;
use mysql::Pool;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::string::String;
use std::thread;
use std::time::Duration;

use crate::roosterteeth::Videos;

// Sleep time in ms between requests for dumping the database.
const SLEEP_TIME: u64 = 1500;

const API_EPISODE_URL: &'static str =
    "https://svod-be.roosterteeth.com/api/v1/episodes?per_page=100&order=desc&page=1";

const API_SHOWS_URL: &'static str =
    "https://svod-be.roosterteeth.com/api/v1/shows?per_page=1000&order=desc";

pub struct Cron {
    pool: Pool,
    logfile: File,
}

impl Cron {
    pub fn init<'a>(addr: &str, auth: &str) -> Self {
        let pool = Pool::new(format!("mysql://{}@{}", auth, addr)).unwrap();

        /*
        pool.prep_exec(
            r"CREATE TABLE IF NOT EXISTS roosterteeth.shows (
                `id` INT UNSIGNED NOT NULL,
                `uuid` CHAR(36) NOT NULL COLLATE 'utf8mb4_unicode_ci',
                `date` DATE NOT NULL,
                `title` VARCHAR(200) NOT NULL COLLATE 'utf8mb4_unicode_ci',
                `slug` VARCHAR(200) NOT NULL COLLATE 'utf8mb4_unicode_ci',
                `summary` VARCHAR(1000) NOT NULL COLLATE 'utf8mb4_unicode_ci',
                `channel` VARCHAR(50) NOT NULL COLLATE 'utf8mb4_unicode_ci',
                `link` VARCHAR(500) NOT NULL COLLATE 'utf8mb4_unicode_ci',
                `canonical_link` VARCHAR(500) NOT NULL COLLATE 'utf8mb4_unicode_ci',
                `seasons` SMALLINT UNSIGNED NOT NULL,
                `episodes` SMALLINT UNSIGNED NOT NULL,
                `image` VARCHAR(500) NOT NULL COLLATE 'utf8mb4_unicode_ci',
                UNIQUE INDEX `Index 1` (`uuid`)
            )
            COLLATE='utf8mb4_unicode_ci'
            ENGINE=InnoDB
            ;",
            (),
        )
        .unwrap();
        */

        pool.prep_exec(
            r"CREATE TABLE IF NOT EXISTS roosterteeth.episodes (
                `id` INT UNSIGNED NOT NULL,
                `title` VARCHAR(200) NOT NULL COLLATE 'utf8mb4_unicode_ci',
                `slug` VARCHAR(200) NOT NULL COLLATE 'utf8mb4_unicode_ci',
                `caption` VARCHAR(2000) NOT NULL COLLATE 'utf8mb4_unicode_ci',
                `description` VARCHAR(10000) NOT NULL COLLATE 'utf8mb4_unicode_ci',
                `channel` VARCHAR(50) NOT NULL COLLATE 'utf8mb4_unicode_ci',
                `link` VARCHAR(500) NOT NULL COLLATE 'utf8mb4_unicode_ci',
                `canonical_link` VARCHAR(500) NOT NULL COLLATE 'utf8mb4_unicode_ci',
                `image` VARCHAR(500) COLLATE 'utf8mb4_unicode_ci',
                `show_title` VARCHAR(200) NOT NULL COLLATE 'utf8mb4_unicode_ci',
                `show_slug` VARCHAR(200) NOT NULL COLLATE 'utf8mb4_unicode_ci',
                `season` SMALLINT UNSIGNED NOT NULL,
                `season_slug` VARCHAR(200) NOT NULL COLLATE 'utf8mb4_unicode_ci',
                `release_public` DATETIME NOT NULL,
                `release_sponsor` DATETIME NOT NULL,
                `sponsor_only` BOOLEAN NOT NULL,
                `length` INT UNSIGNED NOT NULL,
                UNIQUE INDEX `Index 1` (`slug`)
            )
            COLLATE='utf8mb4_unicode_ci'
            ENGINE=InnoDB
            ;",
            (),
        )
        .unwrap();

        Cron {
            pool: pool,
            logfile: OpenOptions::new()
                .append(true)
                .create(true)
                .open("/var/log/rtdownloader")
                .unwrap(),
        }
    }

    pub fn run(&mut self) {
        self.log("Cron::run has begun");
        // BEGIN ROOSTERTEETH CRON JOB
        self.cron_update();

        self.log("Cron::run has ended.");
    }

    fn log<S: Into<String>>(&mut self, line: S) {
        // https://stackoverflow.com/a/38957921/1687505
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let out = format!("{} ~ {}", timestamp, line.into());
        println!("{}", out);
        self.logfile.write_all(out.as_bytes()).unwrap();
    }

    // BEGIN MAIN FUNCTIONS

    fn cron_update(&mut self) {
        self.log("Updating RoosterTeeth database.");

        self.process_episodes(API_EPISODE_URL.to_string());
        //self.process_shows();
    }

    // Dangerous function!
    pub fn dump(&mut self) {
        self.log("cron::dump starting...");

        let channels = vec![
            "rooster-teeth",
            "achievement-hunter",
            "funhaus",
            "inside-gaming",
            "screwattack",
            "sugar-pine-7",
            "cow-chop",
            "game-attack",
            "jt-music",
            "kinda-funny",
        ];

        let mut body: Videos;
        let mut pages: u16;

        let sleep_time = Duration::from_millis(SLEEP_TIME);

        for chan in channels.iter() {
            body = ::reqwest::get(&format!("https://svod-be.roosterteeth.com/api/v1/episodes?per_page=100&channel_id={}&order=desc&page=1",chan)).unwrap().json().unwrap();
            pages = body.pagination.total_pages;

            self.log(format!(
                "Dumping RoosterTeeth channel '{}', containing {} pages",
                chan, pages
            ));

            for i in 1..(pages + 1) {
                self.process_episodes(format!("https://svod-be.roosterteeth.com/api/v1/episodes?per_page=100&channel_id={}&order=desc&page={}",chan,i));
                thread::sleep(sleep_time);
            }
        }
        self.log("cron::dump has ended.");
    }

    /*
    // TODO: Refactor into proper structs
    fn process_shows(&self) {
        let body = ::reqwest::get(API_SHOW_URL).unwrap().text().unwrap();

        let d: Value = ::serde_json::from_str(&body).unwrap();

        let mut img: String;
        let mut date: String;

        let blank = String::new();

        for mut stmt in self.pool.prepare("INSERT INTO roosterteeth.shows (id, uuid, date, title, slug, summary, channel, link, canonical_link, seasons, episodes, image) VALUES(:id, :uuid, :date, :title, :slug, :summary, :channel, :link, :canonical_link, :seasons, :episodes, :image) ON DUPLICATE KEY UPDATE date=:date, seasons=:seasons, episodes=:episodes").into_iter() {
            for v in d["data"].as_array().unwrap().iter() {
                img = blank.clone(); // Assign a blank string as initialization of img is 'conditional'
                for i in v["included"]["images"].as_array().unwrap().iter() {
                    if i["attributes"]["image_type"] == "title_card" {
                        img = fix_string(&i["attributes"]["small"]);
                        break;
                    }
                }

                date = fix_string(&v["attributes"]["last_episode_golive_at"]);
                date = String::from(&date[..10]);

                // Same as above, id is stored as 2^24 rather than native rust type 2^32
                stmt.execute(params!{
                    "id" => v["id"].as_u64().unwrap() as u32,
                    "uuid" => fix_string(&v["uuid"]),
                    "date" => date,
                    "title" => fix_string(&v["attributes"]["title"]),
                    "slug" => fix_string(&v["attributes"]["slug"]),
                    "summary" => fix_string(&v["attributes"]["summary"]),
                    "channel" => fix_string(&v["attributes"]["channel_slug"]),
                    "link" => fix_string(&v["links"]["self"]),
                    "canonical_link" => fix_string(&v["canonical_links"]["self"]),
                    "seasons" => v["attributes"]["season_count"].as_u64().unwrap() as u16,
                    "episodes" => v["attributes"]["episode_count"].as_u64().unwrap() as u16,
                    "image" => img,
                }).unwrap();
            }
        }
    }
    */

    fn process_episodes(&mut self, url: String) {
        let body: Videos = ::reqwest::get(&url).unwrap().json().unwrap();

        for mut stmt in self.pool.prepare("INSERT INTO roosterteeth.episodes (id, release_public, release_sponsor, title, slug, caption, description, channel, link, canonical_link, image, show_title, show_slug, season, season_slug, sponsor_only, length) VALUES(:id, :release_public, :release_sponsor, :title, :slug, :caption, :description, :channel, :link, :canonical_link, :image, :show_title, :show_slug, :season, :season_slug, :sponsor_only, :length) ON DUPLICATE KEY UPDATE id=id").into_iter() {
            for v in &body.data {
                stmt.execute(params!{
                    "id" => &v.id,
                    // Weird hack to store ISO8601 value in MariaDB DateTime column.
                    // https://stackoverflow.com/a/11232145
                    "release_public" => &v.attributes.release_public[..16],
                    "release_sponsor" => &v.attributes.release_sponsor[..16],
                    "title" => &v.attributes.title,
                    "slug" => &v.attributes.slug,
                    "caption" => &v.attributes.caption,
                    "description" => &v.attributes.description,
                    "channel" => &v.attributes.channel_slug,
                    "link" => &v.links.own,
                    "canonical_link" => &v.canonical_links.own,
                    "image" => &v.included.images[0].attributes.large,
                    "show_title" => &v.attributes.show_title,
                    "show_slug" => &v.attributes.show_slug,
                    "season" => &v.attributes.season_number,
                    "season_slug" => &v.attributes.season_slug,
                    "sponsor_only" => &v.attributes.is_sponsors_only,
                    "length" => &v.attributes.length,
                }).unwrap();
            }
        }
    }

    // END MAIN FUNCTIONS
}
