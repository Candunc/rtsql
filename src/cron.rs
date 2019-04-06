use chrono::Local;
use mysql::Pool;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::string::String;
use std::thread;
use std::time::Duration;

use crate::roosterteeth::{Shows, Videos};

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

        pool.prep_exec(
            r"CREATE TABLE IF NOT EXISTS roosterteeth.shows (
                `id` INT UNSIGNED NOT NULL,
                `last_update` DATETIME NOT NULL,
                `title` VARCHAR(200) NOT NULL COLLATE 'utf8mb4_unicode_ci',
                `slug` VARCHAR(200) NOT NULL COLLATE 'utf8mb4_unicode_ci',
                `summary` VARCHAR(1000) NOT NULL COLLATE 'utf8mb4_unicode_ci',
                `channel` VARCHAR(50) NOT NULL COLLATE 'utf8mb4_unicode_ci',
                `link` VARCHAR(500) NOT NULL COLLATE 'utf8mb4_unicode_ci',
                `canonical_link` VARCHAR(500) NOT NULL COLLATE 'utf8mb4_unicode_ci',
                `seasons` SMALLINT UNSIGNED NOT NULL,
                `episodes` SMALLINT UNSIGNED NOT NULL,
                `image` VARCHAR(500) COLLATE 'utf8mb4_unicode_ci',
                UNIQUE INDEX `Index 1` (`slug`)
            )
            COLLATE='utf8mb4_unicode_ci'
            ENGINE=InnoDB
            ;",
            (),
        )
        .unwrap();

        pool.prep_exec(
            r"CREATE TABLE IF NOT EXISTS roosterteeth.episodes (
                `id` INT UNSIGNED NOT NULL,
                `title` VARCHAR(200) NOT NULL COLLATE 'utf8mb4_unicode_ci',
                `slug` VARCHAR(200) NOT NULL COLLATE 'utf8mb4_unicode_ci',
                `caption` VARCHAR(2000) NOT NULL COLLATE 'utf8mb4_unicode_ci',
                `description` TEXT NOT NULL COLLATE 'utf8mb4_unicode_ci',
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
        self.process_shows();
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
                "Dumping channel '{}', containing {} pages",
                chan, pages
            ));

            for i in 1..(pages + 1) {
                self.process_episodes(format!("https://svod-be.roosterteeth.com/api/v1/episodes?per_page=100&channel_id={}&order=desc&page={}",chan,i));
                thread::sleep(sleep_time);
            }
        }
        self.log("cron::dump has ended.");
    }

    fn process_shows(&self) {
        let body: Shows = ::reqwest::get(API_SHOWS_URL).unwrap().json().unwrap();

        let mut img: String;

        for mut stmt in self.pool.prepare("INSERT INTO roosterteeth.shows (id, last_update, title, slug, summary, channel, link, canonical_link, seasons, episodes, image) VALUES(:id, :last_update, :title, :slug, :summary, :channel, :link, :canonical_link, :seasons, :episodes, :image) ON DUPLICATE KEY UPDATE last_update=:last_update, seasons=:seasons, episodes=:episodes").into_iter() {
            for v in &body.data {
                img = String::new();
                for i in &v.included.images {
                    if i.attributes.image_type == "title_card" {
                        img = i.attributes.large.clone();
                        break;
                    }
                }

                stmt.execute(params!{
                    "id" => &v.id,
                    "last_update" => &v.attributes.last_update[..16],
                    "title" => &v.attributes.title,
                    "slug" => &v.attributes.slug,
                    "summary" => &v.attributes.summary,
                    "channel" => &v.attributes.channel_slug,
                    "link" => &v.links.own,
                    "canonical_link" => &v.canonical_links.own,
                    "seasons" => &v.attributes.season_count,
                    "episodes" => &v.attributes.episode_count,
                    "image" => &img,
                }).unwrap();
            }
        }
    }

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
