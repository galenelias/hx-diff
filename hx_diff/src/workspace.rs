// Functions for managing the main state of the application
// Including all scanned files, app query parameters, etc.
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::SeqCst;
use git_cli_wrap as git;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProjectEntryId(usize);

impl ProjectEntryId {
	pub const MAX: Self = Self(usize::MAX);

	pub fn new(counter: &AtomicUsize) -> Self {
		Self(counter.fetch_add(1, SeqCst))
	}
}

// pub struct Sha1Hash([u8; 40]);
// pub struct Sha1Hash(String);

pub enum FileSource {
	// TODO, name
	Working,
	Index(git::Sha1Hash),
	Head(git::Sha1Hash),
	Commit(git::Sha1Hash),
}

pub enum CategoryKind {
	Staged,
	Working,
	Commit,
}

pub struct FileEntry {
	path: PathBuf,
	left_source: FileSource,
	right_source: FileSource,
}

pub enum EntryKind {
	Category(CategoryKind),
	File(FileEntry),
	Directory(PathBuf),
}

// File details:
//  LeftState (Staged, )
//
// File State: HEAD, Commit, Index (staging), Working Directory

pub struct Entry {
	id: ProjectEntryId,
	kind: EntryKind,
	path: PathBuf,
	// status: String,
}

pub struct Workspace {
	// entries: HashMap<ProjectEntryId, Entry>,
	entries: Vec<Entry>,
}

impl Workspace {
	fn for_git_status() -> Self {
		let git_status = git::get_status().expect("Failed to get git status");

		// let mut entries = HashMap::new();
		let counter = AtomicUsize::new(0);
		let mut entries = Vec::new();

		let mut process_items = |get_status: fn(&git::Entry) -> git::EntryStatus,
		                         is_staged: bool,
		                         category_name: &'static str| {
			let mut has_items = false;
			let mut last_dir = None;

			for entry in git_status
				.entries
				.iter()
				.filter(|e| get_status(e) != git::EntryStatus::None)
			{
				let path = &entry.path;

				if !has_items {
					entries.push(Entry {
						id: ProjectEntryId::new(&counter),
						kind: if is_staged {
							EntryKind::Category(CategoryKind::Staged)
						} else {
							EntryKind::Category(CategoryKind::Working)
						},
						path: path.clone().into(),
					});
					has_items = true;
				}
				let parent_dir = path.parent().expect(&format!(
					"Failed to get parent directory for '{}'",
					entry.path.display()
				));

				if Some(parent_dir) != last_dir {
					last_dir = Some(parent_dir);
					entries.push(Entry {
						id: ProjectEntryId::new(&counter),
						kind: EntryKind::Directory(parent_dir.into()),
						path: parent_dir.to_owned(),
					});
				}

				// let status = get_status(entry).to_string();
				let file_entry = FileEntry {
					path: path.clone(),
					left_source: if is_staged {
						FileSource::Head(entry.head_sha1)
					} else {
						FileSource::Index(entry.index_sha1)
					},
					right_source: if is_staged {
						FileSource::Index(entry.index_sha1)
					} else {
						FileSource::Working
					},
				};

				entries.push(Entry {
					id: ProjectEntryId::new(&counter),
					kind: EntryKind::File(file_entry),
					path: path.clone(),
				});
			}
		};

		process_items(
			|e| e.staged_status,
			/*is_staged=*/ true,
			"STAGED - Changes to be committed",
		);
		process_items(
			|e| e.unstaged_status,
			/*is_staged=*/ false,
			"WORKING - Changes not staged for commit",
		);

		return Workspace { entries };
	}
}
