use ::std::io::Read;
use ::std::fs::File;
use ::std::string::String;

#[derive(Serialize, Deserialize)]
pub struct Config {
	pub address: String,
	pub userpass: String
}

impl Config {
	pub fn load() -> Self {
		// https://stackoverflow.com/a/27474958/1687505
		let mut file = File::open("/usr/local/etc/rtdownloader/config.json").expect("Install config.json to /usr/local/etc/rtdownloader/ and rerun.");
		let mut contents = String::new();
		file.read_to_string(&mut contents).expect("Unable to read file");

		let output: Config = ::serde_json::from_str(&contents).unwrap();

		output
	}
}
