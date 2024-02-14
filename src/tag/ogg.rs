use chrono::{Datelike, NaiveDate};
use oggvorbismeta::{read_comment_header, replace_comment_header, CommentHeader, VorbisComments};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use super::Field;
use crate::error::SpotifyError;

pub struct OggTag {
	path: PathBuf,
	tag: CommentHeader,
}

impl OggTag {
	/// Load tag from file
	pub fn open(path: impl AsRef<Path>) -> Result<OggTag, SpotifyError> {
		let mut file = File::open(&path)?;
		let tag = read_comment_header(&mut file);
		Ok(OggTag {
			path: path.as_ref().to_owned(),
			tag,
		})
	}
}

impl super::Tag for OggTag {
	fn set_separator(&mut self, _separator: &str) {}

	fn set_field(&mut self, field: Field, value: Vec<String>) {
		let tag = match field {
			Field::Title => "TITLE",
			Field::Artist => "ARTIST",
			Field::Album => "ALBUM",
			Field::TrackNumber => "TRACKNUMBER",
			Field::DiscNumber => "DISCNUMBER",
			Field::Genre => "GENRE",
			Field::Label => "LABEL",
			Field::AlbumArtist => "ALBUMARTIST",
		};
		self.set_raw(tag, value);
	}

	fn add_cover(&mut self, _mime: &str, _data: Vec<u8>) {
		error!("ALBUM ART IN OGG NOT SUPPORTED!");
		match self.path.parent() {
			Some(p) => {
				let cp = PathBuf::from(p).join("cover.jpg");
				match File::create(cp) {
					Ok(mut file) => {
						file.write_all(&_data);
						file.flush();
						info!("saved album art to file");
					},
					Err(e) => {
						error!("Failed to open file {}", e);
					}
				}				
			},
			None => {
				warn!("no parent path")
			}
		}
	}

	fn set_raw(&mut self, tag: &str, value: Vec<String>) {
		self.tag.add_tag_multi(
			tag,
			&value.iter().map(|v| v.as_str()).collect::<Vec<&str>>(),
		);
	}

	fn save(&mut self) -> Result<(), SpotifyError> {
		let file = File::open(&self.path)?;
		let mut out = replace_comment_header(file, self.tag.clone());
		let mut file = File::create(&self.path)?;
		std::io::copy(&mut out, &mut file)?;
		Ok(())
	}

	fn set_release_date(&mut self, date: NaiveDate) {
		self.tag.add_tag_single(
			"DATE",
			&format!("{}-{:02}-{:02}", date.year(), date.month(), date.day()),
		)
	}
}
