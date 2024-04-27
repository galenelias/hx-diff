// Functions for managing the main state of the application
// Including all scanned files, app query parameters, etc.
use crate::Args;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::SeqCst;
use git_cli_wrap as git;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProjectEntryId(usize);

impl ProjectEntryId {
	pub const _MAX: Self = Self(usize::MAX);

	pub fn new(counter: &AtomicUsize) -> Self {
		Self(counter.fetch_add(1, SeqCst))
	}

	pub fn to_usize(&self) -> usize {
		self.0
	}
}

pub enum FileSource {
	// TODO, name
	Empty,
	Working,
	Index(git::Sha1Hash),
	Head(git::Sha1Hash),
	Commit(git::Sha1Hash),
}

impl FileSource {
	fn commit_or_working(sha1: &git::Sha1Hash) -> Self {
		if sha1.is_zero() {
			Self::Working
		} else {
			Self::Commit(*sha1)
		}
	}

	pub fn left_from_entry(entry: &git::ShowEntry) -> Self {
		match entry.status {
			git::FileStatus::Added => Self::Empty,
			git::FileStatus::Modified => Self::Head(entry.left_sha1),
			git::FileStatus::Deleted => Self::Commit(entry.left_sha1),
			_ => panic!("Unhandled"),
		}
	}

	pub fn right_from_entry(entry: &git::ShowEntry) -> Self {
		match entry.status {
			git::FileStatus::Added => Self::commit_or_working(&entry.right_sha1),
			git::FileStatus::Modified => Self::commit_or_working(&entry.right_sha1),
			git::FileStatus::Deleted => Self::Empty,
			_ => panic!("Unhandled"),
		}
	}
}

pub enum CategoryKind {
	Staged,
	Working,
	Commit,
}

pub struct FileEntry {
	pub path: PathBuf,
	pub left_source: FileSource,
	pub right_source: FileSource,
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
	pub id: ProjectEntryId,
	pub kind: EntryKind,
	pub path: PathBuf,
	// status: String,
}

#[derive(PartialEq, Clone)]
pub enum WorkspaceMode {
	GitStatus,
	GitShow(String),
	GitDiff(git::DiffOptions),
}

pub struct Workspace {
	// entries: HashMap<ProjectEntryId, Entry>,
	pub mode: WorkspaceMode,
	pub entries: Vec<Entry>,
}

impl Workspace {
	pub fn get_entry(&self, id: ProjectEntryId) -> Option<&Entry> {
		// TODO: Make this efficient
		self.entries.iter().find(|entry| entry.id == id)
	}

	pub fn from_args(args: &Args) -> Self {
		match args.mode.as_deref() {
			None | Some("status") => Self::for_git_status(),
			Some("diff") => Self::for_git_diff(args),
			Some("show") => Self::for_git_show(
				&args
					.arg
					.as_ref()
					.expect("Missing commit argument for 'show'"),
			),
			_ => panic!("Invalid mode"),
		}
	}

	pub fn for_git_diff(args: &Args) -> Self {
		let diff_options = git::DiffOptions {
			merge_base: args.merge_base,
			cached: args.cached || args.staged,
			commit: args.arg.clone(),
		};

		let git_diff = git::get_diff(&diff_options).expect("Failed to get git diff");

		let counter = AtomicUsize::new(0);
		let mut entries = Vec::new();

		entries.push(Entry {
			id: ProjectEntryId::new(&counter),
			kind: EntryKind::Category(CategoryKind::Commit),
			path: PathBuf::new(),
		});

		for entry in git_diff.entries.iter() {
			let path = &entry.path;
			let left_source = FileSource::left_from_entry(&entry);
			let right_source = FileSource::right_from_entry(&entry);

			entries.push(Entry {
				id: ProjectEntryId::new(&counter),
				kind: EntryKind::File(FileEntry {
					path: path.clone(),
					left_source,
					right_source,
				}),
				path: path.clone(),
			});
		}

		return Workspace {
			mode: WorkspaceMode::GitDiff(diff_options),
			entries,
		};
	}

	pub fn for_git_show(commit: &str) -> Self {
		let git_show = git::show(commit).expect("Failed to get git show");

		let counter = AtomicUsize::new(0);
		let mut entries = Vec::new();

		entries.push(Entry {
			id: ProjectEntryId::new(&counter),
			kind: EntryKind::Category(CategoryKind::Commit),
			path: PathBuf::new(),
		});

		for entry in git_show.entries.iter() {
			let path = &entry.path;
			let left_source = FileSource::Commit(entry.left_sha1);
			let right_source = FileSource::Commit(entry.right_sha1);

			entries.push(Entry {
				id: ProjectEntryId::new(&counter),
				kind: EntryKind::File(FileEntry {
					path: path.clone(),
					left_source,
					right_source,
				}),
				path: path.clone(),
			});
		}

		return Workspace {
			mode: WorkspaceMode::GitShow(commit.to_owned()),
			entries,
		};
	}

	pub fn for_git_status() -> Self {
		let git_status = git::get_status().expect("Failed to get git status");

		let counter = AtomicUsize::new(0);
		let mut entries = Vec::new();

		let mut process_items = |get_status: fn(&git::StatusEntry) -> git::EntryStatus,
		                         is_staged: bool,
		                         _category_name: &'static str| {
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

		return Workspace {
			mode: WorkspaceMode::GitStatus,
			entries,
		};
	}
}
